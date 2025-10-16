mod common;

use {
    alloy_chains::NamedChain,
    cctp_bridge::{Cctp, SolanaWrapper},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    let (owner, rpc) = common::solana_setup()?;
    let rpc: SolanaWrapper = rpc.into();
    let bridge = Cctp::new_recv(
        rpc.clone(),
        rpc,
        NamedChain::BaseSepolia,
        cctp_bridge::SOLANA_DEVNET,
    );
    let result = bridge
        .recv_message_sol(
            &owner,
            "0x1de765f7d19b45913190863d8cd60c1e58e48a85b60b0ff7bf39329076aabd7b",
        )
        .await?;
    println!("{result}");
    Ok(())
}
