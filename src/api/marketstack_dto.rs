use chrono::{DateTime, Local};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Getters, new)]
pub struct MarketstackQuoteDto {
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    adj_high: Decimal,
    adj_low: Decimal,
    adj_close: Decimal,
    adj_open: Decimal,
    adj_volume: Decimal,
    split_factor: Decimal,
    dividend: Decimal,
    name: String,
    exchange_code: String,
    asset_type: String,
    price_currency: String,
    symbol: String,
    exchange: String,
    date: DateTime<Local>,
}

#[derive(Debug, Deserialize, Getters, new)]
pub struct MarketstackSearchSymbolDto {
    name: String,
    symbol: String,
    cik: String,
    isin: String,
    ein_employer_id: String,
    lei: String,
    series_id: String,
    item_type: String,
    sector: String,
    industry: String,
    sic_code: String,
    sic_name: String,
    stock_exchange: String,
}
