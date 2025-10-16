use {
    TokenMessenger::TokenMessengerInstance,
    alloy_network::Ethereum,
    alloy_primitives::{Address, FixedBytes, U256, address},
    alloy_provider::Provider,
    alloy_rpc_types::TransactionRequest,
    alloy_sol_types::sol,
};

// https://developers.circle.com/cctp/evm-smart-contracts
pub const TOKEN_MESSENGER_CONTRACT: Address =
    address!("0x28b5a0e9C621a5BadaA536219b3a228C8168cf5d");
pub const TOKEN_MESSENGER_CONTRACT_TESTNET: Address =
    address!("0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA");
pub const ARBITRUM_TOKEN_MESSENGER_ADDRESS: Address = TOKEN_MESSENGER_CONTRACT;
pub const ARBITRUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS: Address = TOKEN_MESSENGER_CONTRACT_TESTNET;
pub const AVALANCHE_TOKEN_MESSENGER_ADDRESS: Address = TOKEN_MESSENGER_CONTRACT;
pub const BASE_TOKEN_MESSENGER_ADDRESS: Address = TOKEN_MESSENGER_CONTRACT;
pub const BASE_SEPOLIA_TOKEN_MESSENGER_ADDRESS: Address = TOKEN_MESSENGER_CONTRACT_TESTNET;
pub const ETHEREUM_TOKEN_MESSENGER_ADDRESS: Address = TOKEN_MESSENGER_CONTRACT;
pub const ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS: Address = TOKEN_MESSENGER_CONTRACT_TESTNET;
pub const OPTIMISM_TOKEN_MESSENGER_ADDRESS: Address = TOKEN_MESSENGER_CONTRACT;
pub const POLYGON_CCTP_TOKEN_MESSENGER: Address = TOKEN_MESSENGER_CONTRACT;
pub const UNICHAIN_CCTP_TOKEN_MESSENGER: Address = TOKEN_MESSENGER_CONTRACT;

pub const ARBITRUM_USDC_CONTRACT: Address = address!("0xaf88d065e77c8cC2239327C5EDb3A432268e5831");
pub const ARBITRUM_SEPOLIA_USDC_CONTRACT: Address =
    address!("0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d");
pub const AVALANCHE_USDC_CONTRACT: Address = address!("0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E");
pub const BASE_USDC_CONTRACT: Address = address!("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
pub const BASE_SEPOLIA_USDC_CONTRACT: Address =
    address!("0x036CbD53842c5426634e7929541eC2318f3dCF7e");
pub const ETHEREUM_USDC_CONTRACT: Address = address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
pub const ETHEREUM_SEPOLIA_USDC_CONTRACT: Address =
    address!("0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238");
pub const OPTIMISM_USDC_CONTRACT: Address = address!("0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85");
pub const OPTIMISM_SEPOLIA_USDC_CONTRACT: Address =
    address!("0x5fd84259d66Cd46123540766Be93DFE6D43130D7");
pub const POLYGON_USDC_CONTRACT: Address = address!("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359");
pub const UNICHAIN_USDC_CONTRACT: Address = address!("0x078D782b760474a361dDA0AF3839290b0EF57AD6");

/// The CCTP v1 Token Messenger contract.
pub struct TokenMessengerContract<P: Provider<Ethereum>> {
    pub instance: TokenMessengerInstance<P>,
}

impl<P: Provider<Ethereum>> TokenMessengerContract<P> {
    /// Create a new TokenMessengerContract.
    pub fn new(address: Address, provider: P) -> Self {
        Self {
            instance: TokenMessengerInstance::new(address, provider),
        }
    }

    /// Create the transaction request for the `depositForBurn` function.
    ///
    /// Most users will want to use this function instead of the
    /// `deposit_for_burn_call_builder` function. destination_caller:
    /// Address as bytes32 which can call receiveMessage on destination domain.
    /// If set to bytes32(0), any address can call receiveMessage
    /// max_fee: Max fee paid for fast burn, specified in units of burnToken
    /// min_finality_threshold: Minimum finality threshold at which burn will be
    /// attested
    #[allow(clippy::too_many_arguments)]
    pub fn deposit_for_burn_transaction(
        &self,
        _from_address: Address,
        recipient: FixedBytes<32>,
        destination_domain: u32,
        token_address: Address,
        amount: U256,
        destination_caller: Address,
        max_fee: U256,
        min_finality_threshold: u32,
    ) -> TransactionRequest {
        self.instance
            .depositForBurn(
                amount,
                destination_domain,
                recipient,
                token_address,
                destination_caller.into_word(),
                max_fee,
                min_finality_threshold,
            )
            .into_transaction_request()
    }
}

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    TokenMessenger,
    "abis/v2_token_messenger.json"
);
