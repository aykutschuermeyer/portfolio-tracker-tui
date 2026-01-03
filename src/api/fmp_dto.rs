use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::models::{Ticker, ticker::ApiProvider};

#[derive(Debug, Deserialize, Getters, new)]
#[serde(rename_all = "camelCase")]
pub struct FmpQuoteDto {
    symbol: String,
    name: String,
    price: Decimal,
    change_percentage: Decimal,
    change: Decimal,
    volume: i64,
    day_low: Decimal,
    day_high: Decimal,
    year_high: Decimal,
    year_low: Decimal,
    market_cap: Option<i64>,
    price_avg_50: Decimal,
    price_avg_200: Decimal,
    exchange: String,
    open: Decimal,
    previous_close: Decimal,
    timestamp: i64,
}

#[derive(Debug, Deserialize, Getters, new)]
#[serde(rename_all = "camelCase")]
pub struct FmpQuoteHistoryDto {
    symbol: String,
    date: String,
    price: Decimal,
    volume: i64,
}

#[derive(Debug, Deserialize, Getters, new)]
#[serde(rename_all = "camelCase")]
pub struct FmpSearchSymbolDto {
    symbol: String,
    name: String,
    currency: String,
    exchange_full_name: String,
    exchange: String,
}

impl FmpSearchSymbolDto {
    pub fn to_ticker(&self) -> Ticker {
        Ticker::new(
            0,
            0,
            self.symbol.clone(),
            self.name.clone(),
            self.currency.clone(),
            Some(self.exchange.clone()),
            None,
            None,
            ApiProvider::Fmp,
        )
    }
}
