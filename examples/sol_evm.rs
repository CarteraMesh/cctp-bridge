use {
    alloy_chains::NamedChain,
    alloy_provider::{ProviderBuilder, WalletProvider},
    alloy_signer_local::PrivateKeySigner,
    cctp_bridge::{Cctp, SolanSigners, SolanaWrapper},
    solana_commitment_config::CommitmentConfig,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_signer::Signer,
    std::{env, str::FromStr},
    tracing::info,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for better debugging
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let secret_key = env::var("EVM_SECRET").expect("EVM_SECRET not set");
    let wallet = PrivateKeySigner::from_str(&secret_key).expect("Invalid private key");
    let api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY not set");
    let kp_file = env::var("KEYPAIR_FILE").expect("KEYPAIR_FILE environment variable not set");
    let owner = solana_keypair::read_keypair_file(&kp_file)
        .map_err(|e| anyhow::format_err!("unable to load keypair file {kp_file} {e}"))?;

    let base_provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(format!("https://base-sepolia.g.alchemy.com/v2/{api_key}").parse()?);

    info!(
        "solana address {} sends to base address {}",
        owner.pubkey(),
        base_provider.default_signer_address()
    );

    let url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL is not set");
    info!("using RPC {url}");
    let rpc: SolanaWrapper =
        RpcClient::new_with_commitment(url, CommitmentConfig::finalized()).into();

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
