use {
    super::{Cctp, Result},
    crate::CctpChain,
    std::fmt::{Display, Formatter},
    tracing::debug,
};

/// Get USDC transfer fees
/// https://developers.circle.com/api-reference/cctp/all/get-burn-usdc-fees
#[derive(Debug, Default, serde::Deserialize)]
pub struct BurnFee {
    /// The finality threshold, such as block confirmations, used to determine
    /// whether the transfer qualifies as a Fast or Standard Transfer.
    #[serde(rename = "finalityThreshold")]
    pub finality_threshold: u32,

    /// Minimum fees for the transfer, expressed in basis points (bps). For
    /// example, 1 = 0.01%.
    #[serde(rename = "minimumFee")]
    pub min_fee: u32,
}

pub struct Fees(pub Vec<BurnFee>);

impl Fees {
    pub fn source_fees(&self) -> u64 {
        // TODO calculate
        match self.0.len() {
            1 => 4, // self.0[0].min_fee,
            _ => 3,
        }
    }

    pub fn source_finality_threshold(&self) -> u32 {
        match self.0.len() {
            1 => self.0[0].finality_threshold,
            _ => 100, // TODO ????
        }
    }
}
impl Display for BurnFee {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "finality threshold {} minimum fee {}",
            self.finality_threshold, self.min_fee
        )
    }
}

impl Display for Fees {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.len() {
            0 => write!(f, "no fees available"),
            1 => write!(f, "source: {}  destination: None", self.0[0]),
            _ => write!(f, "source: {}  destination: {}", self.0[0], self.0[1]),
        }
    }
}

impl<SrcProvider, DstProvider> Cctp<SrcProvider, DstProvider> {
    pub async fn get_fees(&self) -> Result<Fees> {
        let url = format!(
            "{}/v2/burn/USDC/fees/{}/{}",
            self.api_url(),
            self.source_chain.cctp_domain_id()?,
            self.destination_chain.cctp_domain_id()?
        );
        debug!("getting fees from {url}");
        let response: Vec<BurnFee> = self.client.get(url).send().await?.json().await?;
        Ok(Fees(response))
    }
}
