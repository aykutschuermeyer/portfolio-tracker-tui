use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::api::fmp::get_quote_history;

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

pub async fn get_exchange_rate(
    base_currency: &str,
    transaction_currency: &str,
    transaction_date: &DateTime<Local>,
    client: &Client,
    api_key: &str,
) -> Result<Decimal> {
    if base_currency == transaction_currency {
        return Ok(dec!(1.0));
    }

    let quote_result = get_quote_history(
        &format!("{}{}", base_currency, transaction_currency),
        &transaction_date.format("%Y-%m-%d").to_string(),
        &transaction_date.format("%Y-%m-%d").to_string(),
        client,
        api_key,
    )
    .await?;

    if let Some(first_quote) = quote_result.first() {
        return Ok(dec!(1) / *first_quote.price());
    } else {
        return Err(anyhow::anyhow!("No quote data available"));
    }
}
