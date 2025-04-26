use std::collections::HashMap;

use anyhow::{Context, Error, Result};
use chrono::{DateTime, Local, TimeZone};
use csv::StringRecord;
use reqwest::Client;
use rust_decimal::Decimal;

use crate::{
    api::fmp::{get_quote, search_symbol},
    models::{Asset, AssetType, Ticker, TransactionType},
};

pub fn parse_transaction(
    record: &StringRecord,
    row_idx: usize,
) -> Result<(
    DateTime<Local>,
    TransactionType,
    String,
    Decimal,
    Decimal,
    Decimal,
    String,
)> {
    let date = parse_datetime(&record[0], row_idx)?;
    let transaction_type = parse_transaction_type(&record[1], row_idx)?;
    let symbol = record[2].to_string();
    let quantity = parse_decimal(&record[3], "quantity", row_idx)?;
    let price = parse_decimal(&record[4], "price", row_idx)?;
    let fees = parse_decimal(&record[5], "fees", row_idx)?;
    let broker = record[6].to_string();

    Ok((
        date,
        transaction_type,
        symbol,
        quantity,
        price,
        fees,
        broker,
    ))
}

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

pub fn split_symbol(symbol: &str) -> (String, String) {
    let mut symbol_split = symbol.split('.');
    let standalone_symbol = symbol_split.next().unwrap_or("").to_string();
    let exchange = symbol_split.next().unwrap_or("").to_string();

    (standalone_symbol, exchange)
}

pub async fn fetch_ticker_data(
    symbol: &str,
    exchange: &str,
    client: &Client,
    api_key: &str,
    row_idx: usize,
) -> Result<(Ticker, String)> {
    let search_results = match search_symbol(symbol, exchange, client, api_key).await {
        Ok(results) => {
            if results.is_empty() {
                return Err(Error::msg(format!(
                    "No results found for symbol '{}' on exchange '{}' at row {}",
                    symbol,
                    exchange,
                    row_idx + 1
                )));
            }
            results
        }
        Err(err) => {
            return Err(Error::msg(format!(
                "Failed to find ticker for symbol '{}' on exchange '{}' at row {}: {}",
                symbol,
                exchange,
                row_idx + 1,
                err
            )));
        }
    };

    let currency = search_results[0].currency().to_string();
    let ticker = search_results[0].to_ticker();

    let quotes = match get_quote(ticker.symbol(), client, api_key).await {
        Ok(quotes) => {
            if quotes.is_empty() {
                return Err(Error::msg(format!(
                    "No quotes found for ticker '{}' at row {}",
                    ticker.symbol(),
                    row_idx + 1
                )));
            }
            quotes
        }
        Err(err) => {
            return Err(Error::msg(format!(
                "Failed to get quote for ticker '{}' at row {}: {}",
                ticker.symbol(),
                row_idx + 1,
                err
            )));
        }
    };

    let price_decimal = *quotes[0].price();

    let ticker_with_price = Ticker::new(
        ticker.symbol().to_string(),
        ticker.name().to_string(),
        ticker.currency().to_string(),
        ticker.exchange().to_string(),
        Some(price_decimal),
        Some(Local::now()),
    );

    Ok((ticker_with_price, currency))
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

pub fn update_position_state(
    position_state: &mut HashMap<String, (Decimal, Decimal, Decimal, Decimal)>,
    ticker_symbol: &str,
    transaction_type: &TransactionType,
    quantity: Decimal,
    price: Decimal,
    fees: Decimal,
) -> (Decimal, Decimal, Decimal, Decimal) {
    let (mut cumulative_units, mut cumulative_cost, mut realized_gains, mut dividends_collected) =
        position_state.get(ticker_symbol).cloned().unwrap_or((
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        ));

    match transaction_type {
        TransactionType::Buy => {
            cumulative_units += quantity;
            cumulative_cost += (price * quantity) + fees;
        }
        TransactionType::Sell => {
            if cumulative_units > Decimal::ZERO {
                let avg_cost_per_share = if cumulative_units > Decimal::ZERO {
                    cumulative_cost / cumulative_units
                } else {
                    Decimal::ZERO
                };

                let sell_quantity = quantity.min(cumulative_units);
                let cost_basis = avg_cost_per_share * sell_quantity;
                let proceeds = price * sell_quantity - fees;
                let gain_loss = proceeds - cost_basis;

                realized_gains += gain_loss;
                cumulative_units -= sell_quantity;

                if cumulative_units > Decimal::ZERO {
                    cumulative_cost = avg_cost_per_share * cumulative_units;
                } else {
                    cumulative_cost = Decimal::ZERO;
                }
            } else {
                eprintln!(
                    "Warning: Attempting to sell more shares than owned for {}: sell quantity {}, owned quantity {}",
                    ticker_symbol, quantity, cumulative_units
                );
            }
        }
        TransactionType::Div => {
            dividends_collected += price * quantity;
        }
    }

    position_state.insert(
        ticker_symbol.to_string(),
        (
            cumulative_units,
            cumulative_cost,
            realized_gains,
            dividends_collected,
        ),
    );

    (
        cumulative_units,
        cumulative_cost,
        realized_gains,
        dividends_collected,
    )
}
