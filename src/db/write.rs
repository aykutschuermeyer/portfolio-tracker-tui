use anyhow::Result;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use sqlx::{Row, Sqlite};

use crate::models::{Ticker, Transaction};

pub async fn insert_ticker(ticker: &Ticker, tx: &mut sqlx::Transaction<'_, Sqlite>) -> Result<i64> {
    let asset_id = sqlx::query(
        r#"
        SELECT id FROM assets
        WHERE name = ?
        "#,
    )
    .bind(ticker.asset().name())
    .fetch_one(&mut **tx)
    .await;

    let asset_id = match asset_id {
        Ok(row) => row.get::<i64, _>("id"),
        Err(_) => {
            let asset = ticker.asset();
            sqlx::query(
                r#"
                INSERT INTO assets 
                (name, asset_type, isin, sector, industry) 
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(asset.name())
            .bind(asset.asset_type().to_str())
            .bind(asset.isin())
            .bind(asset.sector())
            .bind(asset.industry())
            .execute(&mut **tx)
            .await?
            .last_insert_rowid()
        }
    };

    let last_price = ticker.last_price().unwrap_or(Decimal::ZERO);
    let id = sqlx::query(
        r#"
        INSERT INTO tickers 
        (symbol, asset_id, currency, exchange, last_price, last_price_updated_at) 
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(ticker.symbol())
    .bind(asset_id)
    .bind(ticker.currency())
    .bind(ticker.exchange())
    .bind(last_price.round_dp(4).to_f64())
    .bind(ticker.last_price_updated_at())
    .execute(&mut **tx)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn insert_transaction(
    transaction: &Transaction,
    ticker_id: &i64,
    tx: &mut sqlx::Transaction<'_, Sqlite>,
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
    .bind(transaction.exchange_rate().round_dp(4).to_f64())
    .bind(transaction.quantity().round_dp(4).to_f64())
    .bind(transaction.price().round_dp(4).to_f64())
    .bind(transaction.fees().round_dp(4).to_f64())
    .bind(position_state.cumulative_units().round_dp(4).to_f64())
    .bind(position_state.cumulative_cost().round_dp(4).to_f64())
    .bind(position_state.cost_of_units_sold().round_dp(4).to_f64())
    .bind(transaction_gains.realized_gains().round_dp(4).to_f64())
    .bind(transaction_gains.dividends_collected().round_dp(4).to_f64())
    .execute(&mut **tx)
    .await?
    .last_insert_rowid();

    Ok(id)
}
