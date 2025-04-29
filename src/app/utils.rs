use anyhow::{Context, Error, Result};
use chrono::{DateTime, Local, TimeZone};
use rust_decimal::Decimal;

use crate::models::{Asset, AssetType, Ticker, TransactionType};

pub fn parse_datetime(field: &str, row_idx: usize) -> Result<DateTime<Local>> {
    let date_str = format!("{} 00:00:00", field);
    let naive = chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")
        .with_context(|| format!("Failed to parse date '{}' at row {}", field, row_idx + 1))?;

    Ok(Local.from_utc_datetime(&naive))
}

pub fn parse_transaction_type(field: &str, row_idx: usize) -> Result<TransactionType> {
    match field {
        "Buy" => Ok(TransactionType::Buy),
        "Sell" => Ok(TransactionType::Sell),
        "Div" => Ok(TransactionType::Div),
        other => Err(Error::msg(format!(
            "Unknown transaction type '{}' at row {}",
            other,
            row_idx + 1
        ))),
    }
}

pub fn parse_decimal(field: &str, field_name: &str, row_idx: usize) -> Result<Decimal> {
    field.parse::<Decimal>().with_context(|| {
        format!(
            "Failed to parse {} '{}' at row {}",
            field_name,
            field,
            row_idx + 1
        )
    })
}

pub fn create_asset(ticker: Ticker) -> Asset {
    Asset::new(
        ticker.name().to_string(),
        AssetType::Stock,
        vec![ticker],
        None,
        None,
        None,
    )
}
