use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use csv::Reader;
use derive_getters::Getters;
use reqwest::Client;
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use rust_decimal_macros::dec;
use sqlx::{Pool, Row, Sqlite};

use crate::{
    api::fmp,
    db::write::{insert_asset, insert_ticker, insert_transaction},
    models::{Asset, AssetType, Holding, Ticker, Transaction, TransactionType},
};

use super::{
    calc::{calculate_position_state, calculate_transaction_gains},
    utils::{get_exchange_rate, parse_datetime, parse_decimal},
};

#[derive(Clone, Debug, Getters)]
pub struct Portfolio {
    base_currency: String,
    connection: Pool<Sqlite>,
    holdings: Vec<Holding>,
    client: Client,
    api_key_av: String,
    api_key_fmp: String,
}

impl Portfolio {
    pub fn new(
        base_currency: String,
        connection: Pool<Sqlite>,
        api_key_av: String,
        api_key_fmp: String,
    ) -> Self {
        Self {
            base_currency,
            connection,
            holdings: Vec::new(),
            client: Client::new(),
            api_key_av,
            api_key_fmp,
        }
    }

    pub async fn set_holdings(&mut self) -> Result<()> {
        let tickers = sqlx::query(
            r#"
            WITH
            cte_transactions_rn AS (
                SELECT 
                    transactions.*,
                    ROW_NUMBER() OVER (PARTITION BY ticker_id, broker ORDER BY transaction_no DESC) AS rn
                FROM
                    transactions
            ),
            cte_transactions AS (
                SELECT
                    *
                FROM
                    cte_transactions_rn
                WHERE
                    rn = 1
            )
            SELECT
                assets.name,
                assets.asset_type,
                assets.isin,
                assets.sector,
                assets.industry,
                tickers.last_price,
                cte_transactions.cumulative_units,
                cte_transactions.cumulative_cost,
                cte_transactions.realized_gains,
                cte_transactions.dividends_collected
            FROM
                tickers
            LEFT JOIN
                assets ON tickers.asset_id = assets.id
            LEFT JOIN
                cte_transactions ON cte_transactions.ticker_id = tickers.id
            "#
        ).fetch_all(&self.connection).await?;

        let holdings: Vec<Holding> = tickers
            .iter()
            .map(|row| {
                let quantity = Decimal::from_f64(row.get::<f64, _>("cumulative_units"))
                    .unwrap_or(Decimal::ZERO);
                let price =
                    Decimal::from_f64(row.get::<f64, _>("last_price")).unwrap_or(Decimal::ZERO);
                let market_value = (price * quantity).round();
                let total_cost = Decimal::from_f64(row.get::<f64, _>("cumulative_cost"))
                    .unwrap_or(Decimal::ZERO);
                let cost_per_share = (total_cost / quantity).round_dp(4);
                let unrealized_gain = market_value - total_cost;
                let unrealized_gain_percent =
                    ((unrealized_gain / total_cost) * dec!(100)).round_dp(2);
                let realized_gain =
                    Decimal::from_f64(row.get::<f64, _>("realized_gains")).unwrap_or(Decimal::ZERO);
                let dividends_collected =
                    Decimal::from_f64(row.get::<f64, _>("dividends_collected"))
                        .unwrap_or(Decimal::ZERO);
                let total_gain = unrealized_gain + realized_gain + dividends_collected;

                Holding::new(
                    Asset::new(
                        row.get::<String, _>("name"),
                        AssetType::parse_str(&row.get::<String, _>("asset_type"))
                            .unwrap_or(AssetType::Stock),
                        row.get::<Option<String>, _>("isin"),
                        row.get::<Option<String>, _>("sector"),
                        row.get::<Option<String>, _>("industry"),
                    ),
                    quantity,
                    price,
                    market_value,
                    total_cost,
                    cost_per_share,
                    unrealized_gain,
                    unrealized_gain_percent,
                    realized_gain,
                    dividends_collected,
                    total_gain,
                )
            })
            .collect();

        self.holdings.clear();
        self.holdings = holdings;

        Ok(())
    }

    async fn get_existing_tickers(&mut self) -> Result<HashMap<String, (Ticker, i64, i64)>> {
        let tickers = sqlx::query(
            r#"
            SELECT * FROM tickers
            LEFT JOIN assets ON tickers.asset_id = assets.id
            "#,
        )
        .fetch_all(&self.connection)
        .await?;

        let mut ticker_map: HashMap<String, (Ticker, i64, i64)> = HashMap::new();
        for row in tickers {
            let symbol: String = row.get::<String, _>("symbol");
            let ticker_id = row.get::<i64, _>("id");
            let asset_id = row.get::<i64, _>("asset_id");
            let ticker = Ticker::new(
                symbol.clone(),
                Asset::new(
                    row.get::<String, _>("name"),
                    AssetType::parse_str(row.get::<&str, _>("asset_type"))?,
                    row.get::<Option<String>, _>("isin"),
                    row.get::<Option<String>, _>("sector"),
                    row.get::<Option<String>, _>("industry"),
                ),
                row.get::<String, _>("currency"),
                row.get::<String, _>("exchange"),
                Decimal::from_f64(row.get::<f64, _>("last_price")),
                row.get::<Option<DateTime<Local>>, _>("last_price_updated_at"),
            );
            ticker_map.insert(symbol, (ticker, ticker_id, asset_id));
        }

        Ok(ticker_map)
    }

    async fn get_existing_forex(&mut self) -> Result<HashMap<i64, Decimal>> {
        let transaction_forex =
            sqlx::query("SELECT transaction_no, exchange_rate FROM transactions")
                .fetch_all(&self.connection)
                .await?;

        let mut forex_map: HashMap<i64, Decimal> = HashMap::new();
        for row in transaction_forex {
            let txn_no = row.get::<i64, _>("transaction_no");
            let x_rate =
                Decimal::from_f64(row.get::<f64, _>("exchange_rate")).unwrap_or(Decimal::ZERO);
            forex_map.insert(txn_no, x_rate);
        }

        Ok(forex_map)
    }

    async fn get_last_transaction_no(&mut self) -> Result<i64> {
        let result =
            sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(transaction_no) FROM transactions")
                .fetch_one(&self.connection)
                .await?;

        Ok(result.unwrap_or(0))
    }

    pub async fn import_transactions(&mut self, path: &str) -> Result<()> {
        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file at path: {}", path))?;

        let mut transactions: Vec<Transaction> = Vec::new();

        let ticker_map = self.get_existing_tickers().await?;
        let forex_map = self.get_existing_forex().await?;

        let _ = self.get_last_transaction_no().await?;

        for (i, record) in reader.records().enumerate() {
            let rec = record.with_context(|| format!("Failed to read CSV record {}", i + 1))?;

            let transaction_no = rec[0].parse::<i64>()?;
            let date = parse_datetime(&rec[1])?;
            let transaction_type = TransactionType::parse_str(&rec[2])?;
            let symbol = rec[3].to_string();
            let quantity = parse_decimal(&rec[4], "quantity")?;
            let price = parse_decimal(&rec[5], "price")?;
            let fees = parse_decimal(&rec[6], "fees")?;
            let broker = rec[7].to_string();

            let symbol: &str = &symbol;
            let mut symbol_split = symbol.split('.');
            let standalone_symbol = symbol_split.next().unwrap_or("").to_string();
            let exchange = symbol_split.next().unwrap_or("").to_string();

            let existing_ticker = ticker_map.get(&standalone_symbol);
            let (ticker, ticker_id, asset_id) = match existing_ticker {
                Some(existing_ticker) => existing_ticker,
                None => {
                    let search_result = fmp::search_symbol(
                        &standalone_symbol,
                        &exchange,
                        &self.client,
                        &self.api_key_fmp,
                    )
                    .await?;
                    &(search_result[0].to_ticker(), 0, 0)
                }
            };

            let currency = ticker.currency();

            let existing_forex = forex_map.get(&transaction_no);
            let exchange_rate = match existing_forex {
                Some(existing_forex) => existing_forex,
                None => {
                    &get_exchange_rate(
                        &self.base_currency,
                        currency,
                        &date,
                        &self.client,
                        &self.api_key_fmp,
                    )
                    .await?
                }
            };

            let mut transaction = Transaction::new(
                transaction_no,
                date,
                transaction_type.clone(),
                ticker.clone(),
                broker.clone(),
                currency.clone(),
                *exchange_rate,
                quantity,
                price,
                fees,
                None,
                None,
            );

            let (mut amounts, mut quantities): (Vec<Decimal>, Vec<Decimal>) = transactions
                .iter()
                .filter(|t| {
                    // t.ticker().asset().name() == ticker.asset().name() && t.broker() == &broker
                    t.ticker().symbol() == ticker.symbol()
                        && t.broker() == &broker
                        && t.currency() == currency
                })
                .map(|t| (t.get_amount(), t.get_quantity()))
                .unzip();

            amounts.push(transaction.get_amount());
            quantities.push(transaction.get_quantity());

            let position_state = calculate_position_state(amounts, quantities)?;
            let transaction_gains = calculate_transaction_gains(&transaction, &position_state);

            transaction.set_position_state(Some(position_state));
            transaction.set_transaction_gains(Some(transaction_gains));

            let mut new_asset_id = *asset_id;
            let mut new_ticker_id = *ticker_id;

            if new_asset_id == 0 {
                new_asset_id = insert_asset(transaction.ticker().asset(), &self.connection).await?;
            }

            if new_ticker_id == 0 {
                new_ticker_id = insert_ticker(ticker, &new_asset_id, &self.connection).await?;
            }

            let _ = insert_transaction(&transaction, &new_ticker_id, &self.connection).await?;

            transactions.push(transaction);
        }

        Ok(())
    }

    pub async fn update_prices(&self) -> Result<()> {
        let tickers = sqlx::query("SELECT symbol FROM tickers")
            .fetch_all(&self.connection)
            .await?;

        for row in tickers {
            let symbol = row.get::<&str, _>("symbol");
            let quote = fmp::get_quote(symbol, &self.client, &self.api_key_fmp).await?;
            let price = *quote[0].price();

            sqlx::query(
                r#"
                UPDATE tickers 
                SET last_price = ?, last_price_updated_at = DATETIME('now') 
                WHERE symbol = ?
                "#,
            )
            .bind(price.to_f64())
            .bind(symbol)
            .execute(&self.connection)
            .await?;
        }

        Ok(())
    }
}
