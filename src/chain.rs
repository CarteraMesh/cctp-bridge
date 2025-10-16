use {
    crate::{
        ARBITRUM_DOMAIN_ID,
        ARBITRUM_MESSAGE_TRANSMITTER_ADDRESS,
        ARBITRUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
        AVALANCHE_DOMAIN_ID,
        AVALANCHE_MESSAGE_TRANSMITTER_ADDRESS,
        BASE_DOMAIN_ID,
        BASE_MESSAGE_TRANSMITTER_ADDRESS,
        BASE_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
        BASE_TOKEN_MESSENGER_ADDRESS,
        ETHEREUM_DOMAIN_ID,
        ETHEREUM_MESSAGE_TRANSMITTER_ADDRESS,
        ETHEREUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
        ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
        ETHEREUM_TOKEN_MESSENGER_ADDRESS,
        OPTIMISM_DOMAIN_ID,
        OPTIMISM_MESSAGE_TRANSMITTER_ADDRESS,
        OPTIMISM_TOKEN_MESSENGER_ADDRESS,
        POLYGON_CCTP_MESSAGE_TRANSMITTER,
        POLYGON_CCTP_TOKEN_MESSENGER,
        POLYGON_DOMAIN_ID,
        SOLANA_DEVNET_ID,
        SOLANA_DEVNET_USDC_TOKEN,
        SOLANA_DOMAIN_ID,
        SOLANA_MAINNET_ID,
        SOLANA_MAINNET_USDC_TOKEN,
        UNICHAIN_CCTP_MESSAGE_TRANSMITTER,
        UNICHAIN_CCTP_TOKEN_MESSENGER,
        UNICHAIN_DOMAIN_ID,
        address::Address,
        error::{Error, Result},
        token_messenger::*,
    },
    alloy_chains::{Chain, ChainKind, NamedChain},
};

/// Trait for chains that support CCTP bridging
pub trait CctpChain {
    /// The average time to confirmation of the chain, according to the CCTP docs: <https://developers.circle.com/stablecoins/required-block-confirmations>
    fn confirmation_average_time_seconds(&self) -> Result<u64>;
    /// The domain ID of the chain - used to identify the chain when bridging: <https://developers.circle.com/stablecoins/evm-smart-contracts>
    fn cctp_domain_id(&self) -> Result<u32>;
    /// The address of the `TokenMessenger` contract on the chain
    fn token_messenger_address(&self) -> Result<Address>;
    /// The address of the `MessageTransmitter` contract on the chain
    fn message_transmitter_address(&self) -> Result<Address>;

    /// Check if the chain is supported for CCTP
    fn is_supported(&self) -> bool;

    fn sandbox(&self) -> bool;

    fn usdc_token_address(&self) -> Result<Address>;
}

impl CctpChain for Chain {
    fn sandbox(&self) -> bool {
        match self.kind() {
            ChainKind::Named(n) => n.sandbox(),
            ChainKind::Id(id) => match *id {
                SOLANA_DEVNET_ID => true,
                SOLANA_MAINNET_ID => false,
                _ => false,
            },
        }
    }

    fn confirmation_average_time_seconds(&self) -> Result<u64> {
        match self.kind() {
            ChainKind::Named(n) => n.confirmation_average_time_seconds(),
            ChainKind::Id(id) => match *id {
                SOLANA_DEVNET_ID => Ok(4),
                SOLANA_MAINNET_ID => Ok(4),
                _ => Err(Error::ChainNotSupported {
                    chain: self.to_string(),
                }),
            },
        }
    }

    fn cctp_domain_id(&self) -> Result<u32> {
        match self.kind() {
            ChainKind::Named(n) => n.cctp_domain_id(),
            ChainKind::Id(id) => match *id {
                SOLANA_DEVNET_ID => Ok(SOLANA_DOMAIN_ID),
                SOLANA_MAINNET_ID => Ok(SOLANA_DOMAIN_ID),
                _ => Err(Error::ChainNotSupported {
                    chain: self.to_string(),
                }),
            },
        }
    }

    fn token_messenger_address(&self) -> Result<Address> {
        match self.kind() {
            ChainKind::Named(n) => n.token_messenger_address(),
            ChainKind::Id(id) => match *id {
                SOLANA_DEVNET_ID | SOLANA_MAINNET_ID => {
                    Ok(nitrogen_circle_token_messenger_minter_v2_encoder::id().into())
                }
                _ => Err(Error::ChainNotSupported {
                    chain: self.to_string(),
                }),
            },
        }
    }

    fn message_transmitter_address(&self) -> Result<Address> {
        match self.kind() {
            ChainKind::Named(n) => n.message_transmitter_address(),
            ChainKind::Id(id) => match *id {
                SOLANA_DEVNET_ID | SOLANA_MAINNET_ID => {
                    Ok(nitrogen_circle_message_transmitter_v2_encoder::id().into())
                }
                _ => Err(Error::ChainNotSupported {
                    chain: self.to_string(),
                }),
            },
        }
    }

    fn is_supported(&self) -> bool {
        match self.kind() {
            ChainKind::Named(n) => n.is_supported(),
            ChainKind::Id(id) => match *id {
                SOLANA_DEVNET_ID => true,
                SOLANA_MAINNET_ID => true,
                _ => false,
            },
        }
    }

    fn usdc_token_address(&self) -> Result<Address> {
        match self.kind() {
            ChainKind::Named(n) => n.usdc_token_address(),
            ChainKind::Id(id) => match *id {
                SOLANA_DEVNET_ID => Ok(SOLANA_DEVNET_USDC_TOKEN.into()),
                SOLANA_MAINNET_ID => Ok(SOLANA_MAINNET_USDC_TOKEN.into()),
                _ => Err(Error::ChainNotSupported {
                    chain: self.to_string(),
                }),
            },
        }
    }
}

impl CctpChain for NamedChain {
    fn sandbox(&self) -> bool {
        self.is_testnet()
    }

    fn confirmation_average_time_seconds(&self) -> Result<u64> {
        use NamedChain::*;

        match self {
            Mainnet | Arbitrum | Base | Optimism | Unichain => Ok(19 * 60),
            Avalanche => Ok(20),
            Polygon => Ok(8 * 60),
            // Testnets
            Sepolia => Ok(60),
            ArbitrumSepolia | AvalancheFuji | BaseSepolia | OptimismSepolia | PolygonAmoy => Ok(20),
            _ => Err(Error::ChainNotSupported {
                chain: self.to_string(),
            }),
        }
    }

    fn cctp_domain_id(&self) -> Result<u32> {
        use NamedChain::*;

        match self {
            Arbitrum | ArbitrumSepolia => Ok(ARBITRUM_DOMAIN_ID),
            Avalanche => Ok(AVALANCHE_DOMAIN_ID),
            Base | BaseSepolia => Ok(BASE_DOMAIN_ID),
            Mainnet | Sepolia => Ok(ETHEREUM_DOMAIN_ID),
            Optimism => Ok(OPTIMISM_DOMAIN_ID),
            Polygon => Ok(POLYGON_DOMAIN_ID),
            Unichain => Ok(UNICHAIN_DOMAIN_ID),
            _ => Err(Error::ChainNotSupported {
                chain: self.to_string(),
            }),
        }
    }

    fn token_messenger_address(&self) -> Result<Address> {
        use NamedChain::*;

        let address = match self {
            Arbitrum => ARBITRUM_TOKEN_MESSENGER_ADDRESS,
            ArbitrumSepolia => ARBITRUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Avalanche => AVALANCHE_TOKEN_MESSENGER_ADDRESS,
            Base => BASE_TOKEN_MESSENGER_ADDRESS,
            BaseSepolia => BASE_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Sepolia => ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Mainnet => ETHEREUM_TOKEN_MESSENGER_ADDRESS,
            Optimism => OPTIMISM_TOKEN_MESSENGER_ADDRESS,
            Polygon => POLYGON_CCTP_TOKEN_MESSENGER,
            Unichain => UNICHAIN_CCTP_TOKEN_MESSENGER,
            _ => {
                return Err(Error::ChainNotSupported {
                    chain: self.to_string(),
                });
            }
        };
        Ok(address.into())
    }

    fn message_transmitter_address(&self) -> Result<Address> {
        use NamedChain::*;

        let address_str = match self {
            Arbitrum => ARBITRUM_MESSAGE_TRANSMITTER_ADDRESS,

            Avalanche => AVALANCHE_MESSAGE_TRANSMITTER_ADDRESS,
            Base => BASE_MESSAGE_TRANSMITTER_ADDRESS,
            Mainnet => ETHEREUM_MESSAGE_TRANSMITTER_ADDRESS,
            Optimism => OPTIMISM_MESSAGE_TRANSMITTER_ADDRESS,
            Polygon => POLYGON_CCTP_MESSAGE_TRANSMITTER,
            // Testnets
            ArbitrumSepolia => ARBITRUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
            BaseSepolia => BASE_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
            Sepolia => ETHEREUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
            Unichain => UNICHAIN_CCTP_MESSAGE_TRANSMITTER,
            _ => {
                return Err(Error::ChainNotSupported {
                    chain: self.to_string(),
                });
            }
        };

        Ok(address_str.into())
    }

    fn is_supported(&self) -> bool {
        use NamedChain::*;

        matches!(
            self,
            Mainnet
                | Arbitrum
                | Base
                | Optimism
                | Unichain
                | Avalanche
                | Polygon
                | Sepolia
                | ArbitrumSepolia
                | AvalancheFuji
                | BaseSepolia
                | OptimismSepolia
                | PolygonAmoy
        )
    }

    fn usdc_token_address(&self) -> Result<Address> {
        use NamedChain::*;

        let address: alloy_primitives::Address = match self {
            Mainnet => ETHEREUM_USDC_CONTRACT,
            Arbitrum => ARBITRUM_USDC_CONTRACT,
            Avalanche => AVALANCHE_USDC_CONTRACT,
            Base => BASE_USDC_CONTRACT,
            Optimism => OPTIMISM_USDC_CONTRACT,
            Polygon => POLYGON_USDC_CONTRACT,
            // Testnets
            ArbitrumSepolia => ARBITRUM_SEPOLIA_USDC_CONTRACT,
            Sepolia => ETHEREUM_SEPOLIA_USDC_CONTRACT,
            BaseSepolia => BASE_SEPOLIA_USDC_CONTRACT,
            OptimismSepolia => OPTIMISM_SEPOLIA_USDC_CONTRACT,
            Unichain => UNICHAIN_USDC_CONTRACT,
            _ => {
                return Err(Error::ChainNotSupported {
                    chain: self.to_string(),
                });
            }
        };

        Ok(address.into())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{SOLANA_DEVNET, SOLANA_MAINNET},
        alloy_chains::NamedChain,
        rstest::rstest,
    };

    #[rstest]
    #[case(NamedChain::Mainnet, true)]
    #[case(NamedChain::Arbitrum, true)]
    #[case(NamedChain::Base, true)]
    #[case(NamedChain::Optimism, true)]
    #[case(NamedChain::Unichain, true)]
    #[case(NamedChain::Avalanche, true)]
    #[case(NamedChain::Polygon, true)]
    #[case(NamedChain::Sepolia, true)]
    #[case(NamedChain::ArbitrumSepolia, true)]
    #[case(NamedChain::AvalancheFuji, true)]
    #[case(NamedChain::BaseSepolia, true)]
    #[case(NamedChain::OptimismSepolia, true)]
    #[case(NamedChain::PolygonAmoy, true)]
    #[case(NamedChain::BinanceSmartChain, false)]
    #[case(NamedChain::Fantom, false)]
    fn test_is_supported(#[case] chain: NamedChain, #[case] expected: bool) {
        assert_eq!(chain.is_supported(), expected);
    }

    #[rstest]
    #[case(NamedChain::Mainnet, 19 * 60)]
    #[case(NamedChain::Arbitrum, 19 * 60)]
    #[case(NamedChain::Base, 19 * 60)]
    #[case(NamedChain::Optimism, 19 * 60)]
    #[case(NamedChain::Unichain, 19 * 60)]
    #[case(NamedChain::Avalanche, 20)]
    #[case(NamedChain::Polygon, 8 * 60)]
    #[case(NamedChain::Sepolia, 60)]
    #[case(NamedChain::ArbitrumSepolia, 20)]
    #[case(NamedChain::AvalancheFuji, 20)]
    #[case(NamedChain::BaseSepolia, 20)]
    #[case(NamedChain::OptimismSepolia, 20)]
    #[case(NamedChain::PolygonAmoy, 20)]
    fn test_confirmation_average_time_seconds_supported_chains(
        #[case] chain: NamedChain,
        #[case] expected: u64,
    ) {
        assert_eq!(chain.confirmation_average_time_seconds().unwrap(), expected);
    }

    #[test]
    fn test_confirmation_average_time_seconds_unsupported_chain() {
        let result = NamedChain::BinanceSmartChain.confirmation_average_time_seconds();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::ChainNotSupported { .. }
        ));
    }

    #[rstest]
    #[case(NamedChain::Arbitrum, ARBITRUM_DOMAIN_ID)]
    #[case(NamedChain::ArbitrumSepolia, ARBITRUM_DOMAIN_ID)]
    #[case(NamedChain::Avalanche, AVALANCHE_DOMAIN_ID)]
    #[case(NamedChain::Base, BASE_DOMAIN_ID)]
    #[case(NamedChain::BaseSepolia, BASE_DOMAIN_ID)]
    #[case(NamedChain::Mainnet, ETHEREUM_DOMAIN_ID)]
    #[case(NamedChain::Sepolia, ETHEREUM_DOMAIN_ID)]
    #[case(NamedChain::Optimism, OPTIMISM_DOMAIN_ID)]
    #[case(NamedChain::Polygon, POLYGON_DOMAIN_ID)]
    #[case(NamedChain::Unichain, UNICHAIN_DOMAIN_ID)]
    fn test_cctp_domain_id_supported_chains(#[case] chain: NamedChain, #[case] expected: u32) {
        assert_eq!(chain.cctp_domain_id().unwrap(), expected);
    }

    #[test]
    fn test_cctp_domain_id_unsupported_chain() {
        let result = NamedChain::BinanceSmartChain.cctp_domain_id();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::ChainNotSupported { .. }
        ));
    }

    #[rstest]
    #[case(NamedChain::Arbitrum, ARBITRUM_TOKEN_MESSENGER_ADDRESS)]
    #[case(NamedChain::ArbitrumSepolia, ARBITRUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS)]
    #[case(NamedChain::Avalanche, AVALANCHE_TOKEN_MESSENGER_ADDRESS)]
    #[case(NamedChain::Base, BASE_TOKEN_MESSENGER_ADDRESS)]
    #[case(NamedChain::BaseSepolia, BASE_SEPOLIA_TOKEN_MESSENGER_ADDRESS)]
    #[case(NamedChain::Sepolia, ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS)]
    #[case(NamedChain::Mainnet, ETHEREUM_TOKEN_MESSENGER_ADDRESS)]
    #[case(NamedChain::Optimism, OPTIMISM_TOKEN_MESSENGER_ADDRESS)]
    #[case(NamedChain::Polygon, POLYGON_CCTP_TOKEN_MESSENGER)]
    #[case(NamedChain::Unichain, UNICHAIN_CCTP_TOKEN_MESSENGER)]
    fn test_token_messenger_address_supported_chains(
        #[case] chain: NamedChain,
        #[case] expected_addr: alloy_primitives::Address,
    ) -> anyhow::Result<()> {
        let result: alloy_primitives::Address = chain.token_messenger_address()?.try_into()?;
        assert_eq!(result, expected_addr);
        Ok(())
    }

    #[rstest]
    #[case(SOLANA_DEVNET, SOLANA_DEVNET_USDC_TOKEN)]
    #[case(SOLANA_MAINNET, SOLANA_MAINNET_USDC_TOKEN)]
    fn test_sol_usdc_address(
        #[case] chain: Chain,
        #[case] expected_addr: solana_pubkey::Pubkey,
    ) -> anyhow::Result<()> {
        let result: solana_pubkey::Pubkey = chain.usdc_token_address()?.try_into()?;
        assert_eq!(result, expected_addr);
        Ok(())
    }

    #[rstest]
    #[case(SOLANA_DEVNET, nitrogen_circle_token_messenger_minter_v2_encoder::ID)]
    #[case(SOLANA_MAINNET, nitrogen_circle_token_messenger_minter_v2_encoder::ID)]
    fn test_sol_token_messenger_address_supported_chains(
        #[case] chain: Chain,
        #[case] expected_addr: solana_pubkey::Pubkey,
    ) -> anyhow::Result<()> {
        let result: solana_pubkey::Pubkey = chain.token_messenger_address()?.try_into()?;
        assert_eq!(result, expected_addr);
        Ok(())
    }

    #[rstest]
    #[case(SOLANA_DEVNET, nitrogen_circle_message_transmitter_v2_encoder::ID)]
    #[case(SOLANA_MAINNET, nitrogen_circle_message_transmitter_v2_encoder::ID)]
    fn test_sol_messenger_address_supported_chains(
        #[case] chain: Chain,
        #[case] expected_addr: solana_pubkey::Pubkey,
    ) -> anyhow::Result<()> {
        let result: solana_pubkey::Pubkey = chain.message_transmitter_address()?.try_into()?;
        assert_eq!(result, expected_addr);
        Ok(())
    }

    #[test]
    fn test_token_messenger_address_unsupported_chain() {
        let result = NamedChain::BinanceSmartChain.token_messenger_address();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::ChainNotSupported { .. }
        ));
    }

    #[rstest]
    #[case(NamedChain::Arbitrum, ARBITRUM_MESSAGE_TRANSMITTER_ADDRESS)]
    #[case(NamedChain::Avalanche, AVALANCHE_MESSAGE_TRANSMITTER_ADDRESS)]
    #[case(NamedChain::Base, BASE_MESSAGE_TRANSMITTER_ADDRESS)]
    #[case(NamedChain::Mainnet, ETHEREUM_MESSAGE_TRANSMITTER_ADDRESS)]
    #[case(NamedChain::Optimism, OPTIMISM_MESSAGE_TRANSMITTER_ADDRESS)]
    #[case(NamedChain::Polygon, POLYGON_CCTP_MESSAGE_TRANSMITTER)]
    #[case(
        NamedChain::ArbitrumSepolia,
        ARBITRUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS
    )]
    #[case(NamedChain::BaseSepolia, BASE_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS)]
    #[case(NamedChain::Sepolia, ETHEREUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS)]
    #[case(NamedChain::Unichain, UNICHAIN_CCTP_MESSAGE_TRANSMITTER)]
    fn test_message_transmitter_address_supported_chains(
        #[case] chain: NamedChain,
        #[case] expected_addr: alloy_primitives::Address,
    ) -> anyhow::Result<()> {
        let result: alloy_primitives::Address =
            chain.message_transmitter_address().unwrap().try_into()?;
        assert_eq!(result, expected_addr);
        Ok(())
    }

    #[test]
    fn test_message_transmitter_address_unsupported_chain() {
        let result = NamedChain::BinanceSmartChain.message_transmitter_address();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::ChainNotSupported { .. }
        ));
    }

    #[test]
    fn test_address_parsing_validation() {
        // All addresses should be valid Ethereum addresses
        for chain in [
            NamedChain::Mainnet,
            NamedChain::Arbitrum,
            NamedChain::Base,
            NamedChain::Optimism,
            NamedChain::Unichain,
            NamedChain::Avalanche,
            NamedChain::Polygon,
            NamedChain::Sepolia,
            NamedChain::ArbitrumSepolia,
            NamedChain::BaseSepolia,
        ] {
            assert!(
                chain.token_messenger_address().is_ok(),
                "Token messenger address should be valid for {chain:?}"
            );
            assert!(
                chain.message_transmitter_address().is_ok(),
                "Message transmitter address should be valid for {chain:?}"
            );
        }
    }
}
