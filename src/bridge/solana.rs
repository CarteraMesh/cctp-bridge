use {
    super::Cctp,
    crate::{
        CctpChain,
        ERC20,
        Error,
        MessageTransmitter,
        Result,
        TokenMessengerContract,
        bridge::recv,
    },
    alloy_chains::NamedChain,
    alloy_network::{Ethereum, NetworkWallet},
    alloy_primitives::{Address as EvmAddress, ruint::aliases::U256},
    alloy_provider::{Provider, WalletProvider},
    nitrogen_circle_token_messenger_minter_v2_encoder::{
        helpers::deposit_for_burn_instruction,
        types::DepositForBurnParams,
    },
    reqwest::Client,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_signature::Signature,
    solana_signer::{Signer, SignerError, signers::Signers},
    std::{sync::Arc, time::Duration},
    tracing::{Level, debug, info, instrument},
};

pub struct SolanSigners<T: Signer> {
    pub owner: T,
    pub message_sent_event_account: Keypair,
    pub rent_payer: Option<T>,
    pub fee_payer: Option<T>,
}

impl<T: Signer> SolanSigners<T> {
    pub fn new(owner: T) -> Self {
        Self {
            fee_payer: None,
            rent_payer: None,
            message_sent_event_account: Keypair::new(),
            owner,
        }
    }
}

impl<T: Signer> Signers for SolanSigners<T> {
    fn pubkeys(&self) -> Vec<Pubkey> {
        vec![
            self.owner.pubkey(),
            self.message_sent_event_account.pubkey(),
        ]
    }

    fn try_pubkeys(&self) -> std::result::Result<Vec<Pubkey>, SignerError> {
        Ok(self.pubkeys())
    }

    fn sign_message(&self, _message: &[u8]) -> Vec<Signature> {
        todo!("not implemented")
    }

    fn try_sign_message(&self, message: &[u8]) -> std::result::Result<Vec<Signature>, SignerError> {
        let mut results = vec![
            self.owner.try_sign_message(message)?,
            self.message_sent_event_account.try_sign_message(message)?,
        ];
        if let Some(ref s) = self.fee_payer {
            results.push(s.try_sign_message(message)?);
        }
        Ok(results)
    }

    fn is_interactive(&self) -> bool {
        true
    }
}

impl From<RpcClient> for SolanaWrapper {
    fn from(rpc: RpcClient) -> Self {
        SolanaWrapper(Arc::new(rpc))
    }
}

#[derive(Clone)]
pub struct SolanaWrapper(Arc<RpcClient>);

pub trait SolanaProvider {
    fn rpc(&self) -> &RpcClient;
}

impl SolanaProvider for SolanaWrapper {
    fn rpc(&self) -> &RpcClient {
        &self.0
    }
}

// Solana to EVM bridging implementation
impl<SrcProvider: SolanaProvider, DstProvider: Provider<Ethereum> + WalletProvider + Clone>
    Cctp<SrcProvider, DstProvider>
{
    pub fn new_solana_evm(
        source_provider: SrcProvider,
        destination_provider: DstProvider,
        source_chain: alloy_chains::Chain,
        destination_chain: NamedChain,
    ) -> Self {
        let recipient = destination_provider.wallet().default_signer_address();
        Self {
            source_provider,
            destination_provider,
            source_chain,
            destination_chain: destination_chain.into(),
            recipient: recipient.into(),
            client: Client::new(),
        }
    }

    #[instrument(skip(self,signers,max_fee,destination_caller,min_finality_threshold), level = Level::INFO
    )]
    pub async fn bridge_sol_evm<S: Signer>(
        &self,
        lamports: u64,
        signers: SolanSigners<S>,
        destination_caller: Option<Pubkey>,
        max_fee: Option<u64>,
        min_finality_threshold: Option<u32>,
        //        attestation_poll_interval: Option<u64>,
    ) -> Result<super::SolanaEvmBridgeResult> {
        info!("burning {lamports}");
        let source_provider = self.source_provider();
        let destination_provider = self.destination_provider();
        let recipient: EvmAddress = self.recipient().try_into()?;
        let mint_recipient = Pubkey::new_from_array(recipient.into_word().into());
        let message_transmitter_evm: EvmAddress = self
            .destination_chain()
            .message_transmitter_address()?
            .try_into()?;
        let destination_domain = self.destination_domain_id()?;
        let usdc_address: Pubkey = self.source_chain().usdc_token_address()?.try_into()?;
        let owner = signers.owner.pubkey();
        let fees = self.get_fees().await?;
        debug!("fees {fees}");
        let params = DepositForBurnParams::builder()
            .amount(lamports)
            .destination_domain(destination_domain)
            .destination_caller(destination_caller.unwrap_or_default())
            .mint_recipient(mint_recipient)
            .max_fee(max_fee.unwrap_or(fees.source_fees()))
            .min_finality_threshold(
                min_finality_threshold.unwrap_or(fees.source_finality_threshold()),
            )
            .build();
        if params.max_fee > lamports {
            return Err(Error::SolanaInvalidFee(params.max_fee, lamports));
        }
        let deposit_for_burn = deposit_for_burn_instruction(
            params,
            owner,
            signers.message_sent_event_account.pubkey(),
            usdc_address,
        );

        info!(
            "using message_sent_event_account {}",
            signers.message_sent_event_account.pubkey()
        );
        let burn_hash = deposit_for_burn
            .tx()
            .send(source_provider.rpc(), Some(&owner), &signers)
            .await?;

        let attestation = self
            .get_attestation_with_retry(burn_hash.to_string(), None, Some(10))
            .await?;

        let message_transmitter =
            MessageTransmitter::new(message_transmitter_evm, destination_provider);

        let recv_message_tx = message_transmitter.receiveMessage(
            attestation.message.clone().into(),
            attestation.attestation.clone().into(),
        );

        info!(
            "recv {lamports} on chain {} recipient {recipient}",
            self.destination_chain(),
        );
        let recv_hash = recv_message_tx
            .send()
            .await?
            .with_required_confirmations(2)
            .with_timeout(Some(Duration::from_secs(90)))
            .watch()
            .await?;

        Ok(super::SolanaEvmBridgeResult {
            attestation,
            burn: burn_hash,
            recv: recv_hash,
        })
    }
}

//  EVM to Solana bridging implementation
impl<SrcProvider: Provider<Ethereum> + WalletProvider + Clone, DstProvider: SolanaProvider>
    Cctp<SrcProvider, DstProvider>
{
    pub fn new_evm_sol(
        source_provider: SrcProvider,
        destination_provider: DstProvider,
        source_chain: NamedChain,
        recipient: Pubkey,
        destination_chain: alloy_chains::Chain,
    ) -> Self {
        Self {
            source_provider,
            destination_provider,
            source_chain: source_chain.into(),
            destination_chain,
            recipient: recipient.into(),
            client: Client::new(),
        }
    }

    #[instrument(skip(self,signer,max_fee,destination_caller,min_finality_threshold), level = Level::INFO
    )]
    pub async fn bridge_evm_sol<S: Signer>(
        &self,
        signer: &S,
        amount: alloy_primitives::U256,
        destination_caller: Option<EvmAddress>,
        max_fee: Option<U256>,
        min_finality_threshold: Option<u32>,
    ) -> Result<super::EvmSolanaBridgeResult> {
        info!("burning {amount}");
        let source_provider = self.source_provider();
        let destination_provider = self.destination_provider();
        let recipient: Pubkey = self.recipient().try_into()?;
        let usdc_sol_address: Pubkey = self.destination_chain().usdc_token_address()?.try_into()?;
        let recipient_usdc_sol_address = spl_associated_token_account::get_associated_token_address(
            &recipient,
            &usdc_sol_address,
        );
        let recipient_bytes32: alloy_primitives::FixedBytes<32> =
            recipient_usdc_sol_address.to_bytes().into();
        let token_messenger: EvmAddress = self.token_messenger_contract()?.try_into()?;
        let destination_domain = self.destination_domain_id()?;
        let usdc_evm_address: EvmAddress = self.source_chain().usdc_token_address()?.try_into()?;
        let usdc_sol_address: Pubkey = self.destination_chain().usdc_token_address()?.try_into()?;
        let erc20 = ERC20::new(usdc_evm_address, source_provider);
        let fees = self.get_fees().await?;
        debug!("fees {fees}");
        let usdc_balance = erc20
            .balanceOf(source_provider.default_signer_address())
            .call()
            .await?;
        debug!("balance {usdc_balance}");

        let current_allowance = erc20
            .allowance(source_provider.default_signer_address(), token_messenger)
            .call()
            .await?;
        if current_allowance < 10 {
            debug!("Approving allowance");
            let approve_hash = erc20
                .approve(token_messenger, U256::from(10))
                .send()
                .await?
                .watch()
                .await?;
            info!("Approved USDC spending: {}", approve_hash);
        }
        let token_messenger = TokenMessengerContract::new(token_messenger, source_provider);
        let burn_tx = token_messenger.deposit_for_burn_transaction(
            source_provider.default_signer_address(),
            recipient_bytes32,
            destination_domain,
            usdc_evm_address,
            amount,
            destination_caller.unwrap_or(EvmAddress::ZERO),
            max_fee.unwrap_or(U256::from(3)),
            min_finality_threshold.unwrap_or(0),
        );

        let burn_hash = source_provider
            .send_transaction(burn_tx)
            .await?
            .with_required_confirmations(2)
            .with_timeout(Some(Duration::from_secs(
                self.source_chain().confirmation_average_time_seconds()?,
            )))
            .watch()
            .await?;
        let attestation = self
            .get_attestation_with_retry(
                format!("0x{}", alloy_primitives::hex::encode(burn_hash)),
                None,
                Some(10),
            )
            .await?;

        let recv_hash = recv::recv_message_internal(
            signer,
            destination_provider.rpc(),
            attestation.clone(),
            &usdc_sol_address,
            usdc_evm_address,
            self.source_chain().cctp_domain_id()?.to_string(),
        )
        .await?;

        Ok(super::EvmSolanaBridgeResult {
            attestation,
            burn: burn_hash,
            recv: recv_hash,
        })
    }
}
