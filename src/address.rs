use {crate::error::Error, alloy_primitives::FixedBytes, std::fmt::Display};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub struct Address(pub FixedBytes<64>, pub usize);

impl Address {
    pub fn new(value: impl Into<String>) -> Self {
        // For backward compatibility with string-based construction
        let s = value.into();
        if let Ok(addr) = s.parse::<alloy_primitives::Address>() {
            Self::from(addr)
        } else if let Ok(pubkey) = s.parse::<solana_pubkey::Pubkey>() {
            Self::from(pubkey)
        } else {
            panic!("Invalid address format: {}", s)
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..self.1]
    }
}

impl AsRef<Address> for Address {
    fn as_ref(&self) -> &Address {
        self
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.1 {
            20 => {
                // EVM address - format as hex with 0x prefix
                let addr = alloy_primitives::Address::from_slice(&self.0[..20]);
                write!(f, "{:#x}", addr)
            }
            solana_pubkey::PUBKEY_BYTES => {
                // Solana address - format as base58
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(&self.0[..32]);
                let pubkey = solana_pubkey::Pubkey::new_from_array(bytes);
                write!(f, "{}", pubkey)
            }
            _ => {
                // Generic hex format for other lengths
                write!(f, "0x{}", alloy_primitives::hex::encode(self.as_bytes()))
            }
        }
    }
}

impl TryFrom<Address> for alloy_primitives::Address {
    type Error = Error;

    fn try_from(addr: Address) -> Result<Self, Self::Error> {
        if addr.1 == 20 {
            Ok(alloy_primitives::Address::from_slice(&addr.0[..20]))
        } else {
            Err(Error::AddrError(format!(
                "Invalid length for EVM address: expected 20, got {}",
                addr.1
            )))
        }
    }
}

impl TryFrom<&Address> for alloy_primitives::Address {
    type Error = Error;

    fn try_from(addr: &Address) -> Result<Self, Self::Error> {
        if addr.1 == 20 {
            Ok(alloy_primitives::Address::from_slice(&addr.0[..20]))
        } else {
            Err(Error::AddrError(format!(
                "Invalid length for EVM address: expected 20, got {}",
                addr.1
            )))
        }
    }
}

impl TryFrom<Address> for solana_pubkey::Pubkey {
    type Error = Error;

    fn try_from(addr: Address) -> Result<Self, Self::Error> {
        Self::try_from(&addr)
        // if addr.1 == solana_pubkey::PUBKEY_BYTES {
        // let mut bytes = [0u8; 32];
        // bytes.copy_from_slice(&addr.0[..32]);
        // Ok(solana_pubkey::Pubkey::new_from_array(bytes))
        // } else {
        // Err(CctpError::AddrError(format!(
        // "Invalid length for Solana address: expected {}, got {} ({addr})",
        // solana_pubkey::PUBKEY_BYTES,
        // addr.1
        // )))
        // }
    }
}

impl TryFrom<&Address> for solana_pubkey::Pubkey {
    type Error = Error;

    fn try_from(addr: &Address) -> Result<Self, Self::Error> {
        if addr.1 == solana_pubkey::PUBKEY_BYTES {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&addr.0[..32]);
            Ok(solana_pubkey::Pubkey::new_from_array(bytes))
        } else {
            Err(Error::AddrError(format!(
                "Invalid length for Solana address: expected {}, got {} ({addr})",
                solana_pubkey::PUBKEY_BYTES,
                addr.1
            )))
        }
    }
}

impl From<alloy_primitives::Address> for Address {
    fn from(addr: alloy_primitives::Address) -> Self {
        let mut bytes = FixedBytes::<64>::ZERO;
        bytes[..20].copy_from_slice(addr.0.as_slice());
        Self(bytes, 20)
    }
}

impl From<solana_pubkey::Pubkey> for Address {
    fn from(pubkey: solana_pubkey::Pubkey) -> Self {
        let mut bytes = FixedBytes::<64>::ZERO;
        bytes[..solana_pubkey::PUBKEY_BYTES].copy_from_slice(&pubkey.to_bytes());
        Self(bytes, solana_pubkey::PUBKEY_BYTES)
    }
}
