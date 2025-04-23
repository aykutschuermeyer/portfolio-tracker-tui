use super::Ticker;
use derive_getters::Getters;
use derive_new::new;

#[derive(Clone, Debug, Getters, new)]
pub struct Asset {
    name: String,
    asset_type: AssetType,
    tickers: Vec<Ticker>,
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
