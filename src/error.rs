use {
    alloy_primitives::{hex::FromHexError, ruint::aliases::U256},
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Chain not supported: {chain}")]
    ChainNotSupported { chain: String },

    #[error("Invalid address: {address}")]
    InvalidAddress {
        address: String,
        #[source]
        source: FromHexError,
    },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Contract call failed: {0}")]
    PendingError(#[from] alloy_provider::PendingTransactionError),

    #[error("Contract call failed: {0}")]
    ContractError(#[from] alloy_contract::Error),

    #[error("Contract call failed: {0}")]
    ContractCall(String),

    #[error("Attestation failed: {reason}")]
    AttestationFailed { reason: String },

    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Timeout waiting for attestation")]
    AttestationTimeout,

    #[error("RPC error: {0}")]
    Rpc(#[from] alloy_json_rpc::RpcError<alloy_transport::TransportErrorKind>),

    #[error("ABI encoding/decoding error: {0}")]
    Abi(#[from] alloy_sol_types::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Hex conversion error: {0}")]
    Hex(#[from] alloy_primitives::hex::FromHexError),

    #[error("No attestation messages found from server")]
    EmptyAttestation,

    #[error("Address conversion error: {0}")]
    AddrError(String),

    #[error("max fee {0} > amount {1}")]
    SolanaInvalidFee(u64, u64),

    #[error(transparent)]
    SolanaSendError(#[from] nitrogen_instruction_builder::Error),

    #[error("failed to get solana claimable accounts: {0}")]
    SolanaClaimableAccountsError(String),

    #[error("failed to get solana fee recipient account: {0}")]
    SolanaFeeRecipientError(String),

    #[error("Insufficient balance have {0} need {1}")]
    InsufficientBalance(U256, U256),
}

pub type Result<T> = std::result::Result<T, Error>;
