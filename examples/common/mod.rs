use {
    alloy_provider::{Provider, ProviderBuilder, WalletProvider},
    alloy_signer_local::PrivateKeySigner,
    solana_commitment_config::CommitmentConfig,
    solana_keypair::Keypair,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_signer::Signer,
    std::{env, str::FromStr},
    tracing::info,
};

#[allow(dead_code)]
pub fn evm_setup() -> anyhow::Result<impl WalletProvider + Provider + Clone> {
    let secret_key = env::var("EVM_SECRET_KEY").expect("EVM_SECRET_KEY not set");
    let wallet = PrivateKeySigner::from_str(&secret_key).expect("Invalid private key");
    let api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY not set");
    let base_provider = ProviderBuilder::new().wallet(wallet).connect_http(
        format!("https://base-sepolia.g.alchemy.com/v2/{api_key}")
            .parse()
            .unwrap(),
    );
    Ok(base_provider)
}
pub fn solana_setup() -> anyhow::Result<(Keypair, RpcClient)> {
    let kp_file = env::var("KEYPAIR_FILE").ok();
    let owner = if let Some(kp) = kp_file {
        solana_keypair::read_keypair_file(&kp)
            .map_err(|e| anyhow::format_err!("unable to load keypair file {kp} {e}"))?
    } else {
        let kp = env::var("TEST_PRIVATE_KEY").expect("TEST_PRIVATE_KEY is not set");
        Keypair::from_base58_string(&kp)
    };
    let url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL is not set");
    info!("using RPC {url}");
    info!("solana address {}", owner.pubkey(),);
    let rpc = RpcClient::new_with_commitment(url, CommitmentConfig::finalized());
    // Your Solana setup code here
    Ok((owner, rpc))
}
