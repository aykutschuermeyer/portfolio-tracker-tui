use super::Ticker;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Asset {
    name: String,
    asset_type: AssetType,
    main_ticker: Ticker,
    other_tickers: Option<Vec<Ticker>>,
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
        main_ticker: Ticker,
        other_tickers: Option<Vec<Ticker>>,
        isin: Option<String>,
        sector: Option<String>,
        industry: Option<String>,
    ) -> Self {
        Self {
            name,
            asset_type,
            isin,
            main_ticker,
            other_tickers,
            sector,
            industry,
        }
    }
}
