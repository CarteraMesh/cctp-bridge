use {
    super::Cctp,
    crate::{Address, Result, SolanaProvider},
    alloy_chains::{Chain, NamedChain},
    nitrogen_circle_message_transmitter_v2_encoder::{
        helpers::reclaim_event_account_helpers,
        instructions::reclaim_event_account,
        types::ReclaimEventAccountParams,
    },
    reqwest::Client,
    solana_pubkey::Pubkey,
    solana_signature::Signature,
    solana_signer::Signer,
    tracing::{debug, info, instrument},
};

impl<SrcProvider: SolanaProvider + Clone, DstProvider: SolanaProvider>
    Cctp<SrcProvider, DstProvider>
{
    pub fn new_reclaim(
        source_provider: SrcProvider,
        destination_provider: DstProvider,
        source_chain: Chain,
    ) -> Self {
        Self {
            source_provider,
            destination_provider,
            source_chain,
            destination_chain: NamedChain::Mainnet.into(), // does not matter
            recipient: Address::default(),
            client: Client::new(),
        }
    }

    #[instrument(skip(self, signer), level = "debug")]
    pub async fn reclaim<T: Signer + ?Sized>(
        &self,
        signer: &T,
    ) -> Result<Vec<(Signature, Pubkey)>> {
        let mut results = Vec::new();
        let rpc = self.source_provider.rpc();
        let reclaim_accounts =
            reclaim_event_account_helpers::find_claimable_accounts(&signer.pubkey(), rpc)
                .await
                .map_err(|err| crate::Error::SolanaClaimableAccountsError(err.to_string()))?;
        for account in reclaim_accounts.accounts {
            debug!("{account}");
            if !account.is_claimable() {
                continue;
            }
            if account.signature.is_none() {
                tracing::warn!("Skipping account {account} with no signature");
                continue;
            }
            let signature = account.signature.unwrap_or_default();
            let attestation = self
                .get_attestation_with_retry(signature, None, None)
                .await?;
            let reclaim_account = reclaim_event_account(
                ReclaimEventAccountParams::builder()
                    .attestation(attestation.attestation)
                    .destination_message(attestation.message)
                    .build(),
            );
            let reclaim_tx = reclaim_account
                .accounts(signer.pubkey(), account.address)
                .tx();
            let sig = reclaim_tx
                .send(rpc, Some(&signer.pubkey()), &[&signer])
                .await?;
            info!("processed: {sig} {}", account.address);
            results.push((sig, account.address));
        }

        Ok(results)
    }
}
