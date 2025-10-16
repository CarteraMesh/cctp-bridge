use solana_pubkey::{Pubkey, pubkey};

pub const SOLANA_DEVNET: alloy_chains::Chain =
    alloy_chains::Chain::from_id_unchecked(SOLANA_DEVNET_ID);
pub const SOLANA_MAINNET: alloy_chains::Chain =
    alloy_chains::Chain::from_id_unchecked(SOLANA_MAINNET_ID);
/// Reowned Chain ID for Solana Devnet
pub(crate) const SOLANA_DEVNET_ID: u64 = chain_id_from_reown!("EtWTRABZaYq6iMfeYKouRu166VU2xqa1");
/// Reowned Chain ID for Solana Mainnet
pub(crate) const SOLANA_MAINNET_ID: u64 = chain_id_from_reown!("5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp");
pub const SOLANA_DEVNET_USDC_TOKEN: Pubkey =
    pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
pub const SOLANA_MAINNET_USDC_TOKEN: Pubkey =
    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
