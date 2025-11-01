use anyhow::{Result, anyhow};
use chrono::{DateTime, Local};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::models::{Ticker, ticker::ApiProvider};

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
    stock_exchange: StockExchange,
}

impl MarketstackSearchSymbolDto {
    pub fn to_ticker(&self) -> Result<Ticker> {
        Ok(Ticker::new(
            self.symbol.clone(),
            self.name.clone(),
            get_currency_from_country_code(&self.stock_exchange().country_code)?,
            Some(self.stock_exchange().acronym().clone()),
            None,
            None,
            ApiProvider::Marketstack,
        ))
    }
}

#[derive(Debug, Deserialize, Getters, new)]
pub struct StockExchange {
    pub name: String,
    pub acronym: String,
    pub mic: String,
    pub country: Option<String>,
    pub country_code: String,
    pub city: String,
    pub website: String,
    pub operating_mic: String,
    pub oprt_sgmt: String,
    pub legal_entity_name: String,
    pub exchange_lei: String,
    pub market_category_code: String,
    pub exchange_status: String,
    pub date_creation: DateInfo,
    pub date_last_update: DateInfo,
    pub date_last_validation: DateInfo,
    pub date_expiry: Option<DateInfo>,
    pub comments: String,
}

#[derive(Debug, Deserialize, Getters, new)]
pub struct DateInfo {
    pub date: String,
    pub timezone_type: i32,
    pub timezone: String,
}

pub fn get_currency_from_country_code(country_code: &str) -> Result<String> {
    match country_code {
        "US" => Ok("USD"),
        "GB" => Ok("GBP"),
        "JP" => Ok("JPY"),
        "CN" => Ok("CNY"),
        "HK" => Ok("HKD"),
        "IN" => Ok("INR"),
        "DE" | "FR" | "IT" | "ES" | "NL" | "BE" | "FI" | "AT" | "IE" => Ok("EUR"),
        "CH" => Ok("CHF"),
        "CA" => Ok("CAD"),
        "AU" => Ok("AUD"),
        "KR" => Ok("KRW"),
        "BR" => Ok("BRL"),
        "SE" => Ok("SEK"),
        "SG" => Ok("SGD"),
        "ZA" => Ok("ZAR"),
        "MX" => Ok("MXN"),
        "RU" => Ok("RUB"),
        "SA" => Ok("SAR"),
        "TR" => Ok("TRY"),
        "TW" => Ok("TWD"),
        "ID" => Ok("IDR"),
        "TH" => Ok("THB"),
        "MY" => Ok("MYR"),
        "PL" => Ok("PLN"),
        "NO" => Ok("NOK"),
        "DK" => Ok("DKK"),
        "AE" => Ok("AED"),
        "AR" => Ok("ARS"),
        "CL" => Ok("CLP"),
        "NZ" => Ok("NZD"),
        _ => Err(anyhow!("Failed to map country code {}", country_code)),
    }
    .map(|x| x.to_string())
}
