mod common;

use {
    cctp_client::{Cctp, SolanaWrapper},
    solana_signer::Signer,
    tracing::info,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    let (owner, rpc) = common::solana_setup()?;
    info!("solana address {}", owner.pubkey(),);
    let rpc: SolanaWrapper = rpc.into();

    let bridge = Cctp::new_reclaim(rpc.clone(), rpc, cctp_client::SOLANA_DEVNET);
    let result = bridge.reclaim(&owner).await?;
    println!("reclaimed {} accounts", result.len());
    for (sig, addr) in result {
        println!("reclaimed account {} with signature {}", addr, sig);
    }
    Ok(())
}
