use {
    super::Cctp,
    crate::{Address, Attestation, CctpChain, Result, SolanaProvider},
    alloy_chains::{Chain, NamedChain},
    alloy_primitives::Address as EvmAddress,
    nitrogen_circle_message_transmitter_v2_encoder::helpers::receive_message_helpers,
    nitrogen_circle_token_messenger_minter_v2_encoder::ID as TOKEN_MESSENGER_PROGRAM_ID,
    reqwest::Client,
    solana_pubkey::Pubkey,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_signature::Signature,
    solana_signer::Signer,
    std::fmt::{Debug, Display},
    tracing::{debug, instrument},
};

pub(crate) async fn recv_message_internal<T: Signer + ?Sized>(
    signer: &T,
    rpc: &RpcClient,
    attestation: Attestation,
    sol_usdc_address: &Pubkey,
    evm_usdc_address: EvmAddress,
    source_chain_id: String,
) -> Result<Signature> {
    debug!("recv on solana for {}", signer.pubkey());
    let owner = signer.pubkey();
    let fee_recipient_token_account = receive_message_helpers::fee_recipient_token_account(
        rpc,
        &TOKEN_MESSENGER_PROGRAM_ID,
        sol_usdc_address,
    )
    .await
    .map_err(|err| crate::Error::SolanaClaimableAccountsError(err.to_string()))?;

    let builder = receive_message_helpers::recv_from_attestation(
        owner,
        TOKEN_MESSENGER_PROGRAM_ID,
        attestation.attestation,
        attestation.message,
    );
    let remaining_accounts = receive_message_helpers::remaining_accounts(
        &owner,
        source_chain_id,
        evm_usdc_address.into_word(),
        &TOKEN_MESSENGER_PROGRAM_ID,
        sol_usdc_address,
        &fee_recipient_token_account,
    );

    builder
        .remaining_accounts(remaining_accounts)
        .tx()
        .send(rpc, Some(&owner), &[&signer])
        .await
        .map_err(|e| e.into())
}

impl<SrcProvider: SolanaProvider, DstProvider: SolanaProvider> Cctp<SrcProvider, DstProvider> {
    pub fn new_recv(
        dummy: SrcProvider,
        destination_provider: DstProvider,
        source_chain: NamedChain,
        destination_chain: Chain,
    ) -> Self {
        Self {
            source_provider: dummy,
            destination_provider,
            source_chain: source_chain.into(),
            destination_chain,
            recipient: Address::default(), // does not matter
            client: Client::new(),
        }
    }

    #[instrument(skip(self, signer), level = "debug")]
    pub async fn recv_message_sol<T: Signer + ?Sized>(
        &self,
        signer: &T,
        tx_hash: impl AsRef<str> + Debug + Display,
    ) -> Result<Signature> {
        let attestation = self.get_attestation_with_retry(tx_hash, None, None).await?;
        let destination_provider = self.destination_provider();
        let sol_usdc_address: Pubkey = self.destination_chain().usdc_token_address()?.try_into()?;
        let evm_usdc_address: EvmAddress = self.source_chain().usdc_token_address()?.try_into()?;
        recv_message_internal(
            signer,
            destination_provider.rpc(),
            attestation,
            &sol_usdc_address,
            evm_usdc_address,
            self.source_chain().cctp_domain_id()?.to_string(),
        )
        .await
    }
}
