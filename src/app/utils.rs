use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

use crate::{
    api::{av, fmp, frank, marketstack},
    models::{Ticker, ticker::ApiProvider},
};

pub fn parse_datetime(field: &str) -> Result<DateTime<Local>> {
    let date_str = format!("{} 00:00:00", field);
    let naive = chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")
        .with_context(|| format!("Failed to parse date '{}'", field))?;

    Ok(Local.from_utc_datetime(&naive))
}

pub fn parse_decimal(field: &str, field_name: &str) -> Result<Decimal> {
    field
        .parse::<Decimal>()
        .with_context(|| format!("Failed to parse {} '{}'", field_name, field))
}

pub async fn find_ticker(symbol: &str, client: &Client, api: &ApiProvider) -> Result<Ticker> {
    match api {
        ApiProvider::AlphaVantage => {
            let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")?;
            let av_search_result = av::search_symbol(symbol, client, api_key.as_str())
                .await
                .with_context(|| format!("Alpha Vantage ({})", symbol))?;

            let first = av_search_result
                .first()
                .with_context(|| "Failed to get first value")?;

            Ok(first.to_ticker())
        }
        ApiProvider::Fmp => {
            let api_key = std::env::var("FMP_API_KEY")?;
            let fmp_search_result = fmp::search_symbol(symbol, client, api_key.as_str())
                .await
                .with_context(|| format!("Alpha Vantage ({})", symbol))?;
            let first = fmp_search_result
                .first()
                .with_context(|| "Failed to get first value")?;
            Ok(first.to_ticker())
        }
        ApiProvider::Marketstack => {
            let api_key = std::env::var("MARKETSTACK_API_KEY")?;
            let marketstack_search_result =
                marketstack::search_symbol(symbol, client, api_key.as_str())
                    .await
                    .with_context(|| format!("Marketstack ({})", symbol))?;
            Ok(marketstack_search_result.to_ticker()?)
        }
    }
}

pub async fn get_latest_price(symbol: &str, client: &Client, api: &ApiProvider) -> Result<Decimal> {
    match api {
        ApiProvider::AlphaVantage => {
            let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")?;
            let av_quote_result = av::get_quote(&symbol, &client, api_key.as_str())
                .await
                .with_context(|| format!("Alpha Vantage ({})", &symbol))?;
            Decimal::from_str(av_quote_result.price())
                .with_context(|| format!("Alpha Vantage ({}): Failed to parse price", symbol))
        }
        ApiProvider::Fmp => {
            let api_key = std::env::var("FMP_API_KEY")?;
            let fmp_quote_result = fmp::get_quote(&symbol, &client, &api_key)
                .await
                .with_context(|| format!("FMP ({})", &symbol))?;
            Ok(*fmp_quote_result
                .first()
                .with_context(|| format!("FMP ({}): Failed to get first entry", symbol))?
                .price())
        }
        ApiProvider::Marketstack => {
            let api_key = std::env::var("MARKETSTACK_API_KEY")?;
            let marketstack_quote_result =
                marketstack::get_quote(&symbol, &client, api_key.as_str()).await?;
            let first = marketstack_quote_result
                .first()
                .with_context(|| "Failed to get first entry")?;
            Ok(*first.close())
        }
    }
}

pub async fn get_exchange_rate(
    base_currency: &str,
    transaction_currency: &str,
    transaction_date: &DateTime<Local>,
    client: &Client,
) -> Result<Decimal> {
    if base_currency == transaction_currency {
        return Ok(dec!(1.0));
    }
    let quote_result = frank::get_forex_history(
        transaction_currency,
        base_currency,
        &transaction_date.format("%Y-%m-%d").to_string(),
        client,
    )
    .await?;
    Ok(quote_result.rates()[base_currency])
}
