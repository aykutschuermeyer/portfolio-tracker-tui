use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use rust_decimal::{Decimal, prelude::FromPrimitive, prelude::ToPrimitive};
use sqlx::{Pool, Row, Sqlite, sqlite::SqliteRow};

use crate::models::{Asset, PositionState, Ticker, Transaction, TransactionGains, TransactionType};

pub async fn insert_ticker(
    ticker: &Ticker,
    asset: &Asset,
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<i64> {
    let asset_id = sqlx::query(
        r#"
        SELECT id FROM assets
        WHERE name = ?
        "#,
    )
    .bind(asset.name())
    .fetch_one(&mut **tx)
    .await;

    let asset_id = match asset_id {
        Ok(row) => row.get::<i64, _>("id"),
        Err(_) => sqlx::query(
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
        .last_insert_rowid(),
    };

    let last_price = ticker.last_price().unwrap_or(Decimal::ZERO);
    let id = sqlx::query(
        r#"
        INSERT INTO tickers
        (symbol, asset_id, currency, exchange, last_price, last_price_updated_at, api)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(ticker.symbol())
    .bind(asset_id)
    .bind(ticker.currency())
    .bind(ticker.exchange())
    .bind(last_price.round_dp(4).to_f64())
    .bind(ticker.last_price_updated_at())
    .bind(ticker.api().to_str())
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
        .with_context(|| "Missing position state")?;

    let transaction_gains = transaction
        .transaction_gains()
        .as_ref()
        .with_context(|| "Missing transaction gains")?;

    let id = sqlx::query(
        r#"
        INSERT OR IGNORE INTO transactions
        (
            transaction_no,
            transaction_date,
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

pub async fn truncate_tables(connection: &Pool<Sqlite>, clear_assets: bool) -> Result<()> {
    let mut tx = connection.begin().await?;

    sqlx::query("DELETE FROM transactions")
        .execute(&mut *tx)
        .await?;

    if clear_assets {
        sqlx::query("DELETE FROM tickers").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM assets").execute(&mut *tx).await?;
    }

    tx.commit().await?;

    Ok(())
}

pub fn parse_i64_from_row(row: &SqliteRow, column: &str) -> Result<i64> {
    row.try_get::<i64, _>(column)
        .with_context(|| format!("Failed to parse i64 from column '{}'", column))
}

pub fn parse_string_from_row(row: &SqliteRow, column: &str) -> Result<String> {
    row.try_get::<String, _>(column)
        .with_context(|| format!("Failed to parse String from column '{}'", column))
}

pub fn parse_f64_from_row(row: &SqliteRow, column: &str) -> Result<f64> {
    let value: f64 = row
        .try_get(column)
        .with_context(|| format!("Failed to parse f64 from column '{}'", column))?;
    Ok(value)
}

pub fn parse_decimal_from_row(row: &SqliteRow, column: &str) -> Result<Decimal> {
    let value = parse_f64_from_row(row, column)?;
    Decimal::from_f64(value)
        .with_context(|| format!("Failed to convert f64 to Decimal for column '{}'", column))
}

pub fn parse_datetime_from_row(row: &SqliteRow, column: &str) -> Result<DateTime<Local>> {
    let timestamp: i64 = row
        .try_get(column)
        .with_context(|| format!("Failed to parse timestamp from column '{}'", column))?;
    Local.timestamp_opt(timestamp, 0).single().with_context(|| {
        format!(
            "Failed to convert timestamp to DateTime for column '{}'",
            column
        )
    })
}

pub fn parse_transaction_type_from_row(row: &SqliteRow, column: &str) -> Result<TransactionType> {
    let type_str = parse_string_from_row(row, column)?;
    TransactionType::parse_str(&type_str)
        .with_context(|| format!("Failed to parse TransactionType from column '{}'", column))
}

pub fn parse_transaction(row: SqliteRow) -> Result<Transaction> {
    let id = parse_i64_from_row(&row, "id")?;
    let ticker_id = parse_i64_from_row(&row, "ticker_id")?;
    let transaction_no = parse_i64_from_row(&row, "transaction_no")?;
    let date = parse_datetime_from_row(&row, "transaction_date")?;
    let transaction_type = parse_transaction_type_from_row(&row, "transaction_type")?;
    let broker = parse_string_from_row(&row, "broker")?;
    let currency = parse_string_from_row(&row, "currency")?;
    let exchange_rate = parse_decimal_from_row(&row, "exchange_rate")?;
    let quantity = parse_decimal_from_row(&row, "quantity")?;
    let price = parse_decimal_from_row(&row, "price")?;
    let fees = parse_decimal_from_row(&row, "fees")?;

    let cumulative_units = parse_decimal_from_row(&row, "cumulative_units")?;
    let cumulative_cost = parse_decimal_from_row(&row, "cumulative_cost")?;
    let cost_of_units_sold = parse_decimal_from_row(&row, "cost_of_units_sold")?;
    let position_state = PositionState::new(cumulative_units, cumulative_cost, cost_of_units_sold);

    let realized_gains = parse_decimal_from_row(&row, "realized_gains")?;
    let dividends_collected = parse_decimal_from_row(&row, "dividends_collected")?;
    let transaction_gains = TransactionGains::new(realized_gains, dividends_collected);

    Ok(Transaction::new(
        id,
        ticker_id,
        transaction_no,
        date,
        transaction_type,
        broker,
        currency,
        exchange_rate,
        quantity,
        price,
        fees,
        Some(position_state),
        Some(transaction_gains),
    ))
}
