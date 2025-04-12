use super::Ticker;

pub struct Asset {
    name: String,
    asset_type: AssetType,
    isin: String,
    main_ticker: Ticker,
    other_tickers: Vec<Ticker>,
    sector: Option<String>,
    industry: Option<String>,
}

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
        isin: String,
        main_ticker: Ticker,
        other_tickers: Vec<Ticker>,
        asset_type: AssetType,
        sector: Option<String>,
        industry: Option<String>,
    ) -> Self {
        Self {
            name,
            isin,
            main_ticker,
            other_tickers,
            asset_type,
            sector,
            industry,
        }
    }
}
