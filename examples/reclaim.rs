mod common;

use {
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

    let bridge = Cctp::new_reclaim(rpc.clone(), rpc, cctp_bridge::SOLANA_DEVNET);
    let result = bridge.reclaim(&owner).await?;
    println!("reclaimed {} accounts", result.len());
    for (sig, addr) in result {
        println!("reclaimed account {} with signature {}", addr, sig);
    }
    Ok(())
}
