# cctp-client

[![Crates.io](https://img.shields.io/crates/v/cctp-client.svg)](https://crates.io/crates/cctp-client)
[![Docs.rs](https://docs.rs/cctp-client/badge.svg)](https://docs.rs/cctp-client)
[![CI](https://github.com/CarteraMesh/cctp-client/workflows/test/badge.svg)](https://github.com/CarteraMesh/cctp-client/actions)
[![Cov](https://codecov.io/github/CarteraMesh/cctp-client/graph/badge.svg?token=dILa1k9tlW)](https://codecov.io/github/CarteraMesh/cctp-client)

## About

cctp-client is a Rust-based helper library for the Cross-Chain Token Protocol [CCTP](https://developers.circle.com/cctp). It facilitates the transfer of USDC between different blockchain networks.
This crates provides flexible control over the transfer process, allowing users to customize various aspects of the transfer.

This project is a fork of the [cctp-rs](https://github.com/semiotic-ai/cctp-rs) [crate](https://crates.io/crates/cctp-rs)

## Example

```rust

mod common;

use {
    alloy_chains::NamedChain,
    alloy_provider::WalletProvider,
    cctp_client::{Cctp, SolanSigners},
    common::*,
    solana_signer::Signer,
    tracing::info,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    // Setup wallets
    let base_sepolia_wallet_provider = evm_base_setup()?;
    let (solana_keypair, rpc) = solana_setup()?;
    info!(
        "solana address {} sends to base address {}",
        solana_keypair.pubkey(),
        base_sepolia_wallet_provider.default_signer_address()
    );

    // Convenience wrapper for cctp_client::SolanaProvider trait
    let rpc_wrapper: cctp_client::SolanaWrapper = rpc.into();
    // Convenience wrapper for solana_signer::Signer for use of CCTP operations
    let signers = SolanSigners::new(solana_keypair);

    let bridge = Cctp::new_solana_evm(
        rpc_wrapper,
        base_sepolia_wallet_provider,
        cctp_client::SOLANA_DEVNET, // source chain
        NamedChain::BaseSepolia, // destination chain
    );
    // 0.000010 USDC to base sepolia
    let result = bridge.bridge_sol_evm(10, signers, None, None, None).await?;
    println!("Solana burn txHash {}", result.burn);
    println!(
        "Base Receive txHash {}",
        alloy_primitives::hex::encode(result.recv)
    );
    Ok(())
}
```

## Development

### Prerequisites

- **Rust Nightly**: Required for code formatting with advanced features
  ```bash
  rustup install nightly
  ```

### Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/CarteraMesh/cctp-client.git
   cd cctp-client
   ```

2. **Build and test**
   ```bash
   # Build the project
   cargo build

   # Run tests (requires valid Fireblocks credentials in .env)
   cargo test

   # Format code (requires nightly)
   cargo +nightly fmt --all
   ```

### Code Formatting

This project uses advanced Rust formatting features that require nightly:

```bash
# Format all code
cargo +nightly fmt --all

# Check formatting
cargo +nightly fmt --all -- --check
```

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
