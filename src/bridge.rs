use {
    super::MessageTransmitter::MessageSent,
    crate::{
        Address,
        Attestation,
        AttestationResponse,
        AttestationStatus,
        CctpChain,
        error::{Error, Result},
    },
    alloy_chains::{Chain, NamedChain},
    alloy_network::Ethereum,
    alloy_primitives::{
        FixedBytes,
        TxHash,
        hex::{self, encode},
    },
    alloy_provider::Provider,
    alloy_sol_types::SolEvent,
    reqwest::{Client, Response},
    solana_signature::Signature as SolanaSignature,
    std::{
        fmt::{Debug, Display},
        thread::sleep,
        time::Duration,
    },
    tracing::{Level, debug, error, info, instrument, trace},
};

mod evm;
mod fee;
mod reclaim;
mod recv;
mod solana;

pub use {fee::*, solana::*};
/// Circle Iris API environment URLs
///
/// See <https://developers.circle.com/stablecoins/cctp-apis>
pub const IRIS_API: &str = "https://iris-api.circle.com";
pub const IRIS_API_SANDBOX: &str = "https://iris-api-sandbox.circle.com";

/// Default confirmation requirements and timeouts for different chains
pub const DEFAULT_CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes default
pub const CHAIN_CONFIRMATION_CONFIG: &[(NamedChain, u64, Duration)] = &[
    // (Chain, Required Confirmations, Timeout)
    (NamedChain::Mainnet, 2, Duration::from_secs(300)), // 5 mins for Ethereum
    (NamedChain::Arbitrum, 1, Duration::from_secs(120)), // 2 mins for Arbitrum
    (NamedChain::Optimism, 1, Duration::from_secs(120)), // 2 mins for Optimism
    (NamedChain::Polygon, 15, Duration::from_secs(180)), // More confirmations for Polygon
    (NamedChain::Avalanche, 3, Duration::from_secs(120)), // 2 mins for Avalanche
    (NamedChain::BinanceSmartChain, 2, Duration::from_secs(120)), // 2 mins for BNB Chain
    (NamedChain::Base, 1, Duration::from_secs(120)),    // 2 mins for Base
    (NamedChain::Unichain, 1, Duration::from_secs(120)), // 2 mins for Unichain
];

/// Gets the chain-specific confirmation configuration
pub fn get_chain_confirmation_config(chain: &NamedChain) -> (u64, Duration) {
    CHAIN_CONFIRMATION_CONFIG
        .iter()
        .find(|(ch, _, _)| ch == chain)
        .map(|(_, confirmations, timeout)| (*confirmations, *timeout))
        .unwrap_or((1, DEFAULT_CONFIRMATION_TIMEOUT))
}

/// For solana reclaim accounts
// pub fn dummy_provider()

#[derive(Clone, Debug)]
pub struct SolanaEvmBridgeResult {
    pub burn: SolanaSignature,
    pub recv: TxHash,
    pub attestation: Attestation,
}

impl Display for SolanaEvmBridgeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Burn: {}, Receive: {}, Attestation: {}",
            self.burn, self.recv, self.attestation
        )
    }
}

#[derive(Clone, Debug)]
pub struct EvmSolanaBridgeResult {
    pub burn: TxHash,
    pub recv: SolanaSignature,
    pub attestation: Attestation,
}

impl Display for EvmSolanaBridgeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Burn: {}, Receive: {}, Attestation: {}",
            self.burn, self.recv, self.attestation
        )
    }
}

#[derive(Clone, Debug)]
pub struct EvmBridgeResult {
    pub approval: Option<TxHash>,
    pub burn: TxHash,
    pub recv: TxHash,
    pub attestation: crate::Attestation,
}

impl Display for EvmBridgeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Approval: {:?}, Burn: {}, Receive: {}, Attestation: {}",
            self.approval, self.burn, self.recv, self.attestation
        )
    }
}

#[derive(Clone)]
pub struct Cctp<SrcProvider, DstProvider> {
    source_provider: SrcProvider,
    destination_provider: DstProvider,
    source_chain: Chain,
    destination_chain: Chain,
    recipient: Address,
    client: Client,
}

impl<SrcProvider, DstProvider> Debug for Cctp<SrcProvider, DstProvider> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src_domain = self.source_chain.cctp_domain_id().unwrap_or(u32::MAX);
        let dst_domain = self.destination_chain.cctp_domain_id().unwrap_or(u32::MAX);
        write!(
            f,
            "CCTP[{}({})->{}({})]",
            self.source_chain, src_domain, self.destination_chain, dst_domain
        )
    }
}

impl<SrcProvider, DstProvider> Cctp<SrcProvider, DstProvider> {
    /// Returns the CCTP API URL for the current environment
    pub fn api_url(&self) -> &'static str {
        if self.source_chain.sandbox() {
            IRIS_API_SANDBOX
        } else {
            IRIS_API
        }
    }

    /// Returns the source chain
    pub fn source_chain(&self) -> &Chain {
        &self.source_chain
    }

    /// Returns the destination chain
    pub fn destination_chain(&self) -> &Chain {
        &self.destination_chain
    }

    /// Returns the destination domain id
    pub fn destination_domain_id(&self) -> Result<u32> {
        self.destination_chain.cctp_domain_id()
    }

    /// Returns the source provider
    pub fn source_provider(&self) -> &SrcProvider {
        &self.source_provider
    }

    /// Returns the destination provider
    pub fn destination_provider(&self) -> &DstProvider {
        &self.destination_provider
    }

    /// Returns the CCTP token messenger contract, the address of the contract
    /// that handles the deposit and burn of USDC
    pub fn token_messenger_contract(&self) -> Result<Address> {
        self.source_chain.token_messenger_address()
    }

    /// Returns the CCTP message transmitter contract, the address of the
    /// contract that handles the receipt of messages
    pub fn message_transmitter_contract(&self) -> Result<Address> {
        self.destination_chain.message_transmitter_address()
    }

    /// Returns the recipient address
    pub fn recipient(&self) -> &Address {
        &self.recipient
    }

    /// Constructs the Iris API URL for a given message hash
    ///
    /// # Arguments
    ///
    /// * `message_hash` - The message hash to query
    ///
    /// # Returns
    ///
    /// The full URL to query the attestation status
    pub fn iris_api_url(&self, message_hash: impl AsRef<str>) -> String {
        format!(
            "{}/v2/messages/{}?transactionHash={}",
            self.api_url(),
            self.source_chain()
                .cctp_domain_id()
                .expect("Chain is not supported"),
            message_hash.as_ref()
        )
    }

    /// Wrapper call to [`get_attestation_with_retry`] for evm [`TxHash`]
    pub async fn get_attestation_evm(
        &self,
        message_hash: TxHash,
        max_attempts: Option<u32>,
        poll_interval: Option<u64>,
    ) -> Result<Attestation> {
        self.get_attestation_with_retry(
            format!("0x{}", encode(message_hash)),
            max_attempts,
            poll_interval,
        )
        .await
    }

    /// Gets the attestation for a message hash from the CCTP API
    ///
    /// # Arguments
    ///
    /// * `message_hash`: The hash of the message to get the attestation for
    /// * `max_attempts`: Maximum number of polling attempts (default: 30)
    /// * `poll_interval`: Time to wait between polling attempts in seconds
    ///   (default: 60)
    ///
    /// # Returns
    ///
    /// The attestation bytes if successful
    pub async fn get_attestation_with_retry(
        &self,
        message_hash: impl AsRef<str>,
        max_attempts: Option<u32>,
        poll_interval: Option<u64>,
    ) -> Result<Attestation> {
        let max_attempts = max_attempts.unwrap_or(30);
        let poll_interval = poll_interval.unwrap_or(60);

        info!(message_hash = ?message_hash.as_ref(), "Polling for attestation ...");

        let url = self.iris_api_url(message_hash);

        info!(url = ?url, "Attestation URL");

        for attempt in 1..=max_attempts {
            trace!(
                attempt = ?attempt,
                max_attempts = ?max_attempts,
                "Getting attestation ..."
            );
            let response = self.get_attestation(&url).await?;
            trace!(response = ?response);

            trace!(attestation_status = ?response.status());

            // Handle rate limiting
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let secs = 5 * 60;
                debug!(sleep_secs = ?secs, "Rate limit exceeded, waiting before retrying");
                sleep(Duration::from_secs(secs));
                continue;
            }

            // Handle 404 status - treat as pending since the attestation likely doesn't
            // exist yet
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                debug!(
                    attempt = ?attempt,
                    max_attempts = ?max_attempts,
                    poll_interval = ?poll_interval,
                    "Attestation not found (404), waiting before retrying"
                );
                sleep(Duration::from_secs(poll_interval));
                continue;
            }

            // Ensure the response status is successful before trying to parse JSON
            response.error_for_status_ref()?;

            debug!("Decoding attestation response");

            let attestation: AttestationResponse = match response.json::<serde_json::Value>().await
            {
                Ok(attestation) => {
                    debug!(attestation = ?attestation, "Attestation response");
                    serde_json::from_value(attestation)?
                }
                Err(e) => {
                    error!(error = ?e, "Error decoding attestation response");
                    continue;
                }
            };

            if attestation.messages.is_empty() {
                return Err(Error::EmptyAttestation);
            }

            let message = attestation.messages.into_iter().next().unwrap();
            match message.status {
                AttestationStatus::Complete => {
                    let attestation_bytes =
                        message
                            .attestation
                            .ok_or_else(|| Error::AttestationFailed {
                                reason: "Attestation missing".to_string(),
                            })?;

                    // Remove '0x' prefix if present and decode hex
                    let attestation_bytes =
                        if let Some(stripped) = attestation_bytes.strip_prefix("0x") {
                            hex::decode(stripped)
                        } else {
                            hex::decode(&attestation_bytes)
                        }?;
                    let attestation_message =
                        message.message.ok_or_else(|| Error::AttestationFailed {
                            reason: "Attestation message missing".to_string(),
                        })?;

                    let attestation_message =
                        if let Some(stripped) = attestation_message.strip_prefix("0x") {
                            hex::decode(stripped)
                        } else {
                            hex::decode(&attestation_message)
                        }?;
                    debug!("Attestation received successfully");
                    return Ok(Attestation {
                        attestation: attestation_bytes,
                        message: attestation_message,
                    });
                }
                AttestationStatus::Failed => {
                    return Err(Error::AttestationFailed {
                        reason: "Attestation failed".to_string(),
                    });
                }
                AttestationStatus::Pending | AttestationStatus::PendingConfirmations => {
                    debug!(
                        attempt = ?attempt,
                        max_attempts = ?max_attempts,
                        poll_interval = ?poll_interval,
                        "Attestation pending, waiting before retrying"
                    );
                    sleep(Duration::from_secs(poll_interval));
                }
            }
        }

        Err(Error::AttestationTimeout)
    }

    /// Gets the attestation for a message hash from the CCTP API
    ///
    /// # Arguments
    ///
    /// * `client`: The HTTP client to use
    /// * `url`: The URL to get the attestation from
    pub async fn get_attestation(&self, url: &str) -> Result<Response> {
        self.client.get(url).send().await.map_err(Error::Network)
    }
}

// EVM-specific implementations
impl<SrcProvider: Provider<Ethereum> + Clone, DstProvider> Cctp<SrcProvider, DstProvider> {
    /// Gets the `MessageSent` event data from a CCTP bridge transaction
    ///
    /// # Arguments
    ///
    /// * `tx_hash`: The hash of the transaction to get the `MessageSent` event
    ///   for
    ///
    /// # Returns
    ///
    /// Returns the message bytes and its hash
    #[instrument(skip(self), level = Level::INFO)]
    pub async fn get_message_sent_event(
        &self,
        tx_hash: TxHash,
    ) -> Result<(Vec<u8>, FixedBytes<32>)> {
        let tx_receipt = self
            .source_provider
            .get_transaction_receipt(tx_hash)
            .await?;

        if let Some(tx_receipt) = tx_receipt {
            // Calculate the event topic by hashing the event signature
            let message_sent_topic = alloy_primitives::keccak256(b"MessageSent(bytes)");

            let message_sent_log = tx_receipt
                .inner
                .logs()
                .iter()
                .find(|log| {
                    log.topics()
                        .first()
                        .is_some_and(|topic| topic.as_slice() == message_sent_topic)
                })
                .ok_or_else(|| Error::TransactionFailed {
                    reason: "MessageSent event not found".to_string(),
                })?;

            // Decode the log data using the generated event bindings
            let decoded = MessageSent::abi_decode_data(&message_sent_log.data().data)?;

            let message_sent_event = decoded.0.to_vec();
            let message_hash = alloy_primitives::keccak256(&message_sent_event);

            Ok((message_sent_event, message_hash))
        } else {
            return Err(Error::TransactionFailed {
                reason: "Transaction not found".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, alloy_chains::NamedChain, rstest::rstest};

    #[rstest]
    #[case(NamedChain::Mainnet, NamedChain::Arbitrum)]
    #[case(NamedChain::Arbitrum, NamedChain::Base)]
    #[case(NamedChain::Base, NamedChain::Polygon)]
    #[case(NamedChain::Sepolia, NamedChain::ArbitrumSepolia)]
    fn test_cross_chain_compatibility(#[case] source: NamedChain, #[case] destination: NamedChain) {
        // Test that chains are supported
        assert!(source.is_supported());
        assert!(destination.is_supported());

        // Test that we can get domain IDs for supported chains
        assert!(source.cctp_domain_id().is_ok());
        assert!(destination.cctp_domain_id().is_ok());
        assert!(source.token_messenger_address().is_ok());
        assert!(destination.message_transmitter_address().is_ok());
    }

    #[test]
    fn test_unsupported_chain_error() {
        let result = NamedChain::BinanceSmartChain.token_messenger_address();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::ChainNotSupported { .. }
        ));
    }
}
