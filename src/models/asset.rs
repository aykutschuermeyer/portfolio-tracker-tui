use serde::{Deserialize, Serialize};

use super::Ticker;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Asset {
    name: String,
    asset_type: AssetType,
    tickers: Vec<Ticker>,
    isin: Option<String>,
    sector: Option<String>,
    industry: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AssetType {
    Stock,
    Bond,
    ETF,
    MutualFund,
    Crypto,
    PreciousMetals,
    Other,
}

impl Asset {
    pub fn new(
        name: String,
        asset_type: AssetType,
        tickers: Vec<Ticker>,
        isin: Option<String>,
        sector: Option<String>,
        industry: Option<String>,
    ) -> Self {
        Self {
            name,
            asset_type,
            isin,
            tickers,
            sector,
            industry,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn asset_type(&self) -> &AssetType {
        &self.asset_type
    }

    pub fn tickers(&self) -> &Vec<Ticker> {
        &self.tickers
    }
}
