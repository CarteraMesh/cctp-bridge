macro_rules! chain_id_from_reown {
    ($chain_str:literal) => {{
        const fn const_hash(s: &str) -> u64 {
            let bytes = s.as_bytes();
            let mut hash = 0xcbf29ce484222325u64; // FNV offset basis
            let mut i = 0;
            while i < bytes.len() {
                hash ^= bytes[i] as u64;
                hash = hash.wrapping_mul(0x100000001b3u64); // FNV prime
                i += 1;
            }
            hash
        }
        const_hash($chain_str)
    }};
}
mod address;
mod attestation;
mod bridge;
mod chain;
mod domain_id;
mod erc;
mod error;
mod message_transmitter;
mod solana;
mod token_messenger;

pub use {
    address::*,
    attestation::*,
    bridge::*,
    chain::*,
    domain_id::*,
    erc::*,
    error::*,
    message_transmitter::*,
    solana::*,
    token_messenger::*,
};
