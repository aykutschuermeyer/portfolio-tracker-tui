use anyhow::Result;
use derive_getters::Getters;
use derive_new::new;

#[derive(Clone, Debug, Getters, new)]
pub struct Asset {
    name: String,
    asset_type: AssetType,
    isin: Option<String>,
    sector: Option<String>,
    industry: Option<String>,
}

#[derive(Clone, Debug)]
pub enum AssetType {
    Stock,
    Bond,
    ETF,
    MutualFund,
    Crypto,
    PreciousMetals,
    Other,
}

impl AssetType {
    pub fn parse_str(s: &str) -> Result<AssetType> {
        match s {
            "Stock" => Ok(AssetType::Stock),
            "Bond" => Ok(AssetType::Bond),
            "ETF" => Ok(AssetType::ETF),
            "MutualFund" => Ok(AssetType::MutualFund),
            "Crypto" => Ok(AssetType::Crypto),
            "PreciousMetals" => Ok(AssetType::PreciousMetals),
            "Other" => Ok(AssetType::Other),
            _ => Err(anyhow::anyhow!("Unknown asset type")),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            AssetType::Stock => "Stock",
            AssetType::Bond => "Bond",
            AssetType::ETF => "ETF",
            AssetType::MutualFund => "MutualFund",
            AssetType::Crypto => "Crypto",
            AssetType::PreciousMetals => "PreciousMetals",
            AssetType::Other => "Other",
        }
    }
}
