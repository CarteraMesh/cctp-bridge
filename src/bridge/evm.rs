use {
    super::Cctp,
    crate::{
        Attestation,
        CctpChain,
        ERC20,
        EvmBridgeResult,
        MessageTransmitter,
        TokenMessengerContract,
        error::Result,
    },
    alloy_chains::NamedChain,
    alloy_network::Ethereum,
    alloy_primitives::{Address as EvmAddress, TxHash, hex::encode, ruint::aliases::U256},
    alloy_provider::{Provider, WalletProvider},
    reqwest::Client,
    std::time::Duration,
    tracing::{Level, debug, info, instrument},
};
// EVM to EVM bridging implementation
impl<
    SrcProvider: Provider<Ethereum> + WalletProvider + Clone,
    DstProvider: Provider<Ethereum> + WalletProvider + Clone,
> Cctp<SrcProvider, DstProvider>
{
    pub fn new(
        source_provider: SrcProvider,
        destination_provider: DstProvider,
        source_chain: NamedChain,
        destination_chain: NamedChain,
        recipient: alloy_primitives::Address,
    ) -> Self {
        Self {
            source_provider,
            destination_provider,
            source_chain: source_chain.into(),
            destination_chain: destination_chain.into(),
            recipient: recipient.into(),
            client: Client::new(),
        }
    }

    #[instrument(skip(self,max_fee,destination_caller,min_finality_threshold), level = Level::INFO)]
    pub async fn burn(
        &self,
        amount: alloy_primitives::U256,
        destination_caller: Option<EvmAddress>,
        max_fee: Option<U256>,
        min_finality_threshold: Option<u32>,
    ) -> Result<(TxHash, Option<TxHash>)> {
        info!("burning {amount}");
        let source_provider = self.source_provider();
        let recipient: EvmAddress = self.recipient().try_into()?;
        let token_messenger: EvmAddress = self.token_messenger_contract()?.try_into()?;
        let destination_domain = self.destination_domain_id()?;
        let usdc_address = self.source_chain().usdc_token_address()?.try_into()?;
        let erc20 = ERC20::new(usdc_address, source_provider);

        let usdc_balance = erc20
            .balanceOf(source_provider.default_signer_address())
            .call()
            .await?;
        debug!("balance {usdc_balance}");

        let current_allowance = erc20
            .allowance(source_provider.default_signer_address(), token_messenger)
            .call()
            .await?;
        let approval_hash: Option<TxHash> = if current_allowance < U256::from(10) {
            debug!("Approving allowance");
            let approve_hash = erc20
                .approve(token_messenger, U256::from(10))
                .send()
                .await?
                .watch()
                .await?;
            info!("Approved USDC spending: {}", approve_hash);
            Some(approve_hash)
        } else {
            None
        };
        let token_messenger = TokenMessengerContract::new(token_messenger, source_provider);
        let burn_tx = token_messenger.deposit_for_burn_transaction(
            source_provider.default_signer_address(),
            recipient.into_word(),
            destination_domain,
            usdc_address,
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

        Ok((burn_hash, approval_hash))
    }

    #[instrument(skip(self), level = Level::INFO)]
    pub async fn recv(&self, burn_hash: TxHash) -> Result<(Attestation, TxHash)> {
        let destination_provider = self.destination_provider();
        let message_transmitter: EvmAddress = self.message_transmitter_contract()?.try_into()?;
        let attestation = self
            .get_attestation_with_retry(format!("0x{}", encode(burn_hash)), None, Some(10))
            .await?;

        let message_transmitter =
            MessageTransmitter::new(message_transmitter, destination_provider);

        let recv_message_tx = message_transmitter.receiveMessage(
            attestation.message.clone().into(),
            attestation.attestation.clone().into(),
        );

        info!("receiving on chain {}", self.destination_chain(),);
        Ok((
            attestation,
            recv_message_tx
                .send()
                .await?
                .with_required_confirmations(2)
                .with_timeout(Some(Duration::from_secs(90)))
                .watch()
                .await?,
        ))
    }

    #[instrument(skip(self,max_fee,destination_caller,min_finality_threshold), level = Level::INFO)]
    pub async fn bridge(
        &self,
        amount: alloy_primitives::U256,
        destination_caller: Option<EvmAddress>,
        max_fee: Option<U256>,
        min_finality_threshold: Option<u32>,
        //        attestation_poll_interval: Option<u64>,
    ) -> Result<super::EvmBridgeResult> {
        let (burn_hash, approval_hash) = self
            .burn(amount, destination_caller, max_fee, min_finality_threshold)
            .await?;
        let (attestation, recv_hash) = self.recv(burn_hash).await?;

        Ok(EvmBridgeResult {
            approval: approval_hash,
            burn: burn_hash,
            recv: recv_hash,
            attestation,
        })
    }
}
