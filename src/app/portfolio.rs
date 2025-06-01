use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use csv::Reader;
use derive_getters::Getters;
use reqwest::Client;
use rust_decimal::{
    Decimal,
    prelude::{FromPrimitive, ToPrimitive},
};
use rust_decimal_macros::dec;
use sqlx::{Pool, Row, Sqlite};

use crate::{
    api::{av, fmp},
    db::write::{insert_ticker, insert_transaction},
    models::{Asset, AssetType, Holding, Ticker, Transaction, TransactionType},
};

use super::{
    calc::{calculate_position_state, calculate_transaction_gains},
    utils::{find_ticker, get_exchange_rate, parse_datetime, parse_decimal},
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
            cte_realized_gains_dividends AS (
                SELECT
                    ticker_id,
                    broker,
                    SUM(realized_gains) as realized_gains,
                    SUM(dividends_collected) as dividends_collected
                FROM
                    transactions
                GROUP BY
                    broker,
                    ticker_id
            ),
            cte_transactions_rn AS (
                SELECT 
                    transactions.*,
                    ROW_NUMBER() OVER (PARTITION BY ticker_id, broker ORDER BY transaction_no DESC)
                        AS rn
                FROM
                    transactions
                WHERE
                    transaction_type IN ('Buy', 'Sell')
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
                ast.name,
                ast.asset_type,
                ast.isin,
                ast.sector,
                ast.industry,
                tcr.last_price,
                tcr.currency,
                tnx.exchange_rate,
                tnx.cumulative_units,
                tnx.cumulative_cost,
                rld.realized_gains,
                rld.dividends_collected
            FROM
                cte_transactions tnx
            LEFT JOIN
                cte_realized_gains_dividends rld 
                ON tnx.ticker_id = rld.ticker_id 
                AND tnx.broker = rld.broker
            LEFT JOIN
                tickers tcr 
                ON tnx.ticker_id = tcr.id
            LEFT JOIN
                assets ast               
                ON tcr.asset_id = ast.id 
            "#,
        )
        .fetch_all(&self.connection)
        .await?;

        let holdings: Vec<Holding> = tickers
            .iter()
            .map(|row| {
                let asset = Asset::new(
                    row.get::<String, _>("name"),
                    AssetType::parse_str(&row.get::<String, _>("asset_type"))
                        .unwrap_or(AssetType::Stock),
                    row.get::<Option<String>, _>("isin"),
                    row.get::<Option<String>, _>("sector"),
                    row.get::<Option<String>, _>("industry"),
                );

                let quantity = Decimal::from_f64(row.get::<f64, _>("cumulative_units"))
                    .unwrap_or(Decimal::ZERO);
                let price =
                    Decimal::from_f64(row.get::<f64, _>("last_price")).unwrap_or(Decimal::ZERO);
                let exchange_rate =
                    Decimal::from_f64(row.get::<f64, _>("exchange_rate")).unwrap_or(dec!(1));

                let adjusted_price = price * (dec!(1) / exchange_rate);

                let market_value = (adjusted_price * quantity).round();
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
                    asset,
                    quantity,
                    adjusted_price,
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

    async fn get_existing_tickers(&mut self) -> Result<HashMap<String, (Ticker, i64)>> {
        let tickers = sqlx::query(
            r#"
            SELECT * FROM tickers
            LEFT JOIN assets ON tickers.asset_id = assets.id
            "#,
        )
        .fetch_all(&self.connection)
        .await?;

        let mut ticker_map: HashMap<String, (Ticker, i64)> = HashMap::new();
        for row in tickers {
            let symbol: String = row.get::<String, _>("symbol");
            let ticker_id = row.get::<i64, _>("id");
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
                row.get("exchange"),
                Decimal::from_f64(row.get::<f64, _>("last_price")),
                row.get::<Option<DateTime<Local>>, _>("last_price_updated_at"),
            );
            ticker_map.insert(symbol, (ticker, ticker_id));
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

        let mut ticker_map = self.get_existing_tickers().await?;
        let forex_map = self.get_existing_forex().await?;

        let last_transaction_no = self.get_last_transaction_no().await?;

        for (i, record) in reader.records().enumerate() {
            let rec = record.with_context(|| format!("Failed to read CSV record {}", i + 1))?;

            let transaction_no = rec[0].parse::<i64>()?;
            if transaction_no <= last_transaction_no {
                continue;
            }

            let date = parse_datetime(&rec[1])?;
            let transaction_type = TransactionType::parse_str(&rec[2])?;
            let symbol = rec[3].to_string();
            let quantity = parse_decimal(&rec[4], "quantity")?;
            let mut price = parse_decimal(&rec[5], "price")?;
            let fees = parse_decimal(&rec[6], "fees")?;
            let broker = rec[7].to_string();
            let alternative_symbol = rec[8].to_string();
            let transaction_currency = rec[9].to_string();

            let existing_ticker = ticker_map.get(&symbol);
            let (ticker, ticker_id) = match existing_ticker {
                Some(existing_ticker) => existing_ticker,
                None => {
                    let search_result =
                        find_ticker(&symbol, &self.client, &self.api_key_fmp, &self.api_key_av)
                            .await;
                    let ticker = match search_result {
                        Ok(result) => result,
                        Err(_) => {
                            find_ticker(
                                &alternative_symbol,
                                &self.client,
                                &self.api_key_fmp,
                                &self.api_key_av,
                            )
                            .await?
                        }
                    };
                    let new_ticker_id = insert_ticker(&ticker, &self.connection).await?;
                    ticker_map.insert(symbol, (ticker.clone(), new_ticker_id));
                    &(ticker, new_ticker_id)
                }
            };

            let currency = ticker.currency();

            if &transaction_currency != currency {
                let x_rate =
                    get_exchange_rate(&currency, &transaction_currency, &date, &self.client)
                        .await?;
                price = price * x_rate;
            }

            let existing_forex = forex_map.get(&transaction_no);
            let exchange_rate = match existing_forex {
                Some(existing_forex) => existing_forex,
                None => {
                    &get_exchange_rate(currency, &self.base_currency, &date, &self.client).await?
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
                        && (*t.transaction_type() == TransactionType::Buy
                            || *t.transaction_type() == TransactionType::Sell)
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

            let _ = insert_transaction(&transaction, ticker_id, &self.connection).await?;

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
            let fmp_quote_result = fmp::get_quote(symbol, &self.client, &self.api_key_fmp).await;
            let price = match fmp_quote_result {
                Ok(result) => *result[0].price(),
                Err(_error) => {
                    // eprintln!("{}", error);
                    let av_quote = av::get_quote(symbol, &self.client, &self.api_key_av).await;
                    if av_quote.is_err() {
                        Decimal::ZERO
                    } else {
                        Decimal::from_str(av_quote?.price()).unwrap_or(Decimal::ZERO)
                    }
                }
            };

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
