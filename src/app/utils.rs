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
    api_key_fmp: &str,
    api_key_av: &str,
) -> Result<Ticker> {
    let fmp_search_result = fmp::search_symbol(&symbol, &client, &api_key_fmp).await;
    match fmp_search_result {
        Ok(result) => Ok(result[0].to_ticker()),
        Err(error) => {
            eprintln!("{}", error);
            let av_search_result = av::search_symbol(&symbol, &client, &api_key_av).await?;
            Ok(av_search_result[0].to_ticker())
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
