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
        cctp_client::SOLANA_DEVNET,
        NamedChain::BaseSepolia,
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
