use {
    alloy_chains::NamedChain,
    cctp_bridge::{Cctp, SolanaWrapper},
    solana_commitment_config::CommitmentConfig,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    std::env,
    tracing::info,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for better debugging
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let kp_file = env::var("KEYPAIR_FILE").expect("KEYPAIR_FILE environment variable not set");
    let owner = solana_keypair::read_keypair_file(&kp_file)
        .map_err(|e| anyhow::format_err!("unable to load keypair file {kp_file} {e}"))?;
    let url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL is not set");
    info!("using RPC {url}");
    let rpc: SolanaWrapper =
        RpcClient::new_with_commitment(url, CommitmentConfig::finalized()).into();
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
