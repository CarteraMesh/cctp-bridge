use {
    alloy_chains::NamedChain,
    alloy_primitives::U256,
    alloy_provider::{Provider, ProviderBuilder, WalletProvider},
    alloy_signer_local::PrivateKeySigner,
    anyhow::Result,
    cctp_bridge::{Cctp, SolanSigners, SolanaWrapper},
    solana_commitment_config::CommitmentConfig,
    solana_keypair::Keypair,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_signer::Signer,
    std::{env, str::FromStr, sync::Once},
    tracing::info,
    tracing_subscriber::{EnvFilter, fmt::format::FmtSpan},
};

pub static INIT: Once = Once::new();

#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
pub fn setup() {
    INIT.call_once(|| {
        if env::var("CI").is_err() {
            // only load .env if not in CI
            if dotenvy::dotenv_override().is_err() {
                eprintln!("no .env file");
            }
        }
        tracing_subscriber::fmt()
            .with_target(false)
            .with_level(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    });
}

#[allow(dead_code)]
pub fn evm_setup(base_sepolia: bool) -> anyhow::Result<impl WalletProvider + Provider + Clone> {
    let secret_key = env::var("EVM_SECRET_KEY").expect("EVM_SECRET_KEY not set");
    let wallet = PrivateKeySigner::from_str(&secret_key).expect("Invalid private key");
    let api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY not set");
    let url = if base_sepolia {
        "https://base-sepolia.g.alchemy.com/v2"
    } else {
        "https://eth-sepolia.g.alchemy.com/v2"
    };
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(format!("{url}/{api_key}").parse()?);
    info!("evm address {}", provider.default_signer_address());
    Ok(provider)
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

const fn usdc_amount(dollars: u64) -> u64 {
    dollars * 1_000_000 // USDC has 6 decimals
}
#[tokio::test]
async fn test_reclaim() -> Result<()> {
    setup();
    let (owner, rpc) = solana_setup()?;
    let rpc: SolanaWrapper = rpc.into();

    let bridge = Cctp::new_reclaim(rpc.clone(), rpc, cctp_bridge::SOLANA_DEVNET);
    let result = bridge.reclaim(&owner).await?;
    info!("reclaimed {} accounts", result.len());
    for (sig, addr) in result {
        info!("reclaimed account {} with signature {}", addr, sig);
    }
    Ok(())
}

#[tokio::test]
async fn test_burn_too_much() -> Result<()> {
    setup();
    let sepolia_provider = evm_setup(false)?;
    let base_provider = evm_setup(true)?;
    let recipient = base_provider.default_signer_address();

    let bridge = Cctp::new(
        sepolia_provider,
        base_provider,
        NamedChain::Sepolia,
        NamedChain::BaseSepolia,
        recipient,
    );
    let too_much: u64 = usdc_amount(10_000_000_000);
    let result = bridge.burn(U256::from(too_much), None, None, None).await;
    assert!(result.is_err(), "Should fail with insufficient balance");

    let e = result.unwrap_err();
    assert!(matches!(e, cctp_bridge::Error::InsufficientBalance(_, _)));
    println!("error {e}");
    Ok(())
}

#[tokio::test]
async fn test_evm_burn_recv_split() -> Result<()> {
    setup();
    let sepolia_provider = evm_setup(false)?;
    let base_provider = evm_setup(true)?;
    let recipient = base_provider.default_signer_address();

    let bridge = Cctp::new(
        sepolia_provider,
        base_provider,
        NamedChain::Sepolia,
        NamedChain::BaseSepolia,
        recipient,
    );
    let (burn_hash, approval_hash) = bridge.burn(U256::from(15), None, None, None).await?;
    assert!(!burn_hash.is_zero(), "Burn hash should not be zero");
    info!(
        "burn {burn_hash} approval {}",
        approval_hash.unwrap_or_default()
    );
    let attest = bridge.get_attestation_evm(burn_hash, None, None).await?;
    let (recv_attest, recv_hash) = bridge.recv(burn_hash, None, None).await?;
    assert!(!recv_hash.is_zero(), "Receive hash should not be zero");
    assert!(
        !attest.attestation.is_empty(),
        "Attestation should not be empty"
    );
    assert!(!attest.message.is_empty(), "Message should not be empty");
    assert_eq!(recv_attest, attest);
    info!("attest {attest} recv {recv_hash}");
    Ok(())
}

#[tokio::test]
async fn test_evm_sol() -> Result<()> {
    setup();
    let sepolia_provider = evm_setup(false)?;
    let (owner, rpc) = solana_setup()?;
    let rpc: SolanaWrapper = rpc.into();

    let bridge = Cctp::new_evm_sol(
        sepolia_provider,
        rpc,
        NamedChain::Sepolia,
        owner.pubkey(),
        cctp_bridge::SOLANA_DEVNET,
    );
    let result = bridge
        .bridge_evm_sol(&owner, U256::from(10), None, None, None)
        .await?;
    info!("bridge result {}", result);
    Ok(())
}

#[tokio::test]
async fn test_sol_evm() -> Result<()> {
    setup();
    let base_provider = evm_setup(true)?;
    let (owner, rpc) = solana_setup()?;
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

    info!("bridge result {}", result);
    Ok(())
}
