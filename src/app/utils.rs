use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    api::{av, fmp, frank},
    models::Ticker,
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

pub async fn find_ticker(
    symbol: &str,
    client: &Client,
    api_key_fmp: &Option<String>,
    api_key_av: &Option<String>,
) -> Result<Ticker> {
    if let Some(api_key) = api_key_av {
        let av_search_result = av::search_symbol(symbol, client, api_key)
            .await
            .with_context(|| format!("Alpha Vantage ({})", symbol));
        if let Ok(av_search_result) = av_search_result {
            let first = av_search_result.first();
            if let Some(first) = first {
                return Ok(first.to_ticker());
            }
        }
    }
    if let Some(api_key) = api_key_fmp {
        let fmp_search_result = fmp::search_symbol(symbol, client, api_key)
            .await
            .with_context(|| format!("Alpha Vantage ({})", symbol))?;
        let first = fmp_search_result
            .first()
            .with_context(|| "Failed to get first value")?;
        return Ok(first.to_ticker());
    } else {
        return Err(anyhow::anyhow!("Missing API key"));
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
