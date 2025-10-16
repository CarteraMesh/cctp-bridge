mod common;

use {
    alloy_chains::NamedChain,
    alloy_primitives::U256,
    alloy_provider::WalletProvider,
    cctp_bridge::{Cctp, SolanaWrapper},
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

    let bridge = Cctp::new_evm_sol(
        base_provider,
        rpc,
        NamedChain::BaseSepolia,
        owner.pubkey(),
        cctp_bridge::SOLANA_DEVNET,
    );
    let result = bridge
        .bridge_evm_sol(&owner, U256::from(10), None, None, None)
        .await?;
    println!("success {result}");
    Ok(())
}
