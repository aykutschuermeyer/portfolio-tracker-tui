use derive_getters::Getters;
use derive_new::new;
use serde::Deserialize;

use crate::models::{Ticker, ticker::ApiProvider};

#[derive(Debug, Deserialize, Getters, new)]
#[serde(rename_all = "camelCase")]
pub struct AvGlobalQuoteDto {
    #[serde(rename = "01. symbol")]
    symbol: String,
    #[serde(rename = "02. open")]
    open: String,
    #[serde(rename = "03. high")]
    high: String,
    #[serde(rename = "04. low")]
    low: String,
    #[serde(rename = "05. price")]
    price: String,
    #[serde(rename = "06. volume")]
    volume: String,
    #[serde(rename = "07. latest trading day")]
    latest_trading_day: String,
    #[serde(rename = "08. previous close")]
    previous_close: String,
    #[serde(rename = "09. change")]
    change: String,
    #[serde(rename = "10. change percent")]
    change_percent: String,
}

#[derive(Debug, Deserialize, Getters, new)]
#[serde(rename_all = "camelCase")]
pub struct AvSymbolSearchDto {
    #[serde(rename = "1. symbol")]
    symbol: String,
    #[serde(rename = "2. name")]
    name: String,
    #[serde(rename = "3. type")]
    asset_type: String,
    #[serde(rename = "4. region")]
    region: String,
    #[serde(rename = "5. marketOpen")]
    market_open: String,
    #[serde(rename = "6. marketClose")]
    market_close: String,
    #[serde(rename = "7. timezone")]
    timezone: String,
    #[serde(rename = "8. currency")]
    currency: String,
    #[serde(rename = "9. matchScore")]
    match_score: String,
}

impl AvSymbolSearchDto {
    pub fn to_ticker(&self) -> Ticker {
        Ticker::new(
            self.symbol.clone(),
            self.name.clone(),
            self.currency.clone(),
            Some(String::from("")),
            None,
            None,
            ApiProvider::AlphaVantage,
        )
    }
}
