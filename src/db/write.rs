use anyhow::Result;
use sqlx::{Pool, Sqlite};

use crate::models::{Asset, Ticker, Transaction};

pub async fn insert_asset(asset: &Asset, connection: &Pool<Sqlite>) -> Result<i64> {
    let id = sqlx::query(
        r#"
        INSERT OR IGNORE INTO assets 
        (name, asset_type, isin, sector, industry) 
        VALUES (?, ?, ?, ?, ?)
    "#,
    )
    .bind(asset.name())
    .bind(asset.asset_type().to_str())
    .bind(asset.isin())
    .bind(asset.sector())
    .bind(asset.industry())
    .execute(connection)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn insert_ticker(
    ticker: &Ticker,
    asset_id: &i64,
    connection: &Pool<Sqlite>,
) -> Result<i64> {
    let id = sqlx::query(
        r#"
            INSERT OR IGNORE INTO tickers 
            (symbol, asset_id, currency, exchange, last_price, last_price_updated_at) 
            VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(ticker.symbol())
    .bind(asset_id)
    .bind(ticker.currency())
    .bind(ticker.exchange())
    .bind(ticker.last_price().map(|d| d.to_string()))
    .bind(ticker.last_price_updated_at())
    .execute(connection)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn insert_transaction(
    transaction: &Transaction,
    ticker_id: &i64,
    connection: &Pool<Sqlite>,
) -> Result<i64> {
    let position_state = transaction
        .position_state()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing position state"))?;

    let transaction_gains = transaction
        .transaction_gains()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing transaction gains"))?;

    let id = sqlx::query(
        r#"
            INSERT OR IGNORE INTO transactions
            (
                transaction_no,
                date,
                transaction_type,
                ticker_id,
                broker,
                currency,
                exchange_rate,
                quantity,
                price,
                fees,
                cumulative_units,
                cumulative_cost,
                cost_of_units_sold,
                realized_gains,
                dividends_collected
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(transaction.transaction_no())
    .bind(transaction.date())
    .bind(transaction.transaction_type().to_str())
    .bind(ticker_id)
    .bind(transaction.broker())
    .bind(transaction.currency())
    .bind(transaction.exchange_rate().to_string())
    .bind(transaction.quantity().to_string())
    .bind(transaction.price().to_string())
    .bind(transaction.fees().to_string())
    .bind(position_state.cumulative_units().to_string())
    .bind(position_state.cumulative_cost().to_string())
    .bind(position_state.cost_of_units_sold().to_string())
    .bind(transaction_gains.realized_gains().to_string())
    .bind(transaction_gains.dividends_collected().to_string())
    .execute(connection)
    .await?
    .last_insert_rowid();

    Ok(id)
}
