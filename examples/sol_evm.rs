mod common;

use {
    alloy_chains::NamedChain,
    alloy_provider::WalletProvider,
    cctp_bridge::{Cctp, SolanSigners, SolanaWrapper},
    solana_signer::Signer,
    tracing::info,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    let base_provider = common::evm_setup()?;
    let (owner, rpc) = common::solana_setup()?;
    info!(
        "solana address {} sends to base address {}",
        owner.pubkey(),
        base_provider.default_signer_address()
    );

    let rpc: SolanaWrapper = rpc.into();

    let bridge = Cctp::new_solana_evm(
        rpc,
        base_provider,
        cctp_bridge::SOLANA_DEVNET,
        NamedChain::BaseSepolia,
    );
    let result = bridge
        .bridge_sol_evm(10, SolanSigners::new(owner), None, None, None)
        .await?;
    println!("success {result}");
    Ok(())
}
