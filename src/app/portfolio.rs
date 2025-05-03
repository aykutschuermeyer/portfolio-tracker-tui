use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use csv::Reader;
use derive_getters::Getters;
use reqwest::Client;
use rust_decimal::Decimal;
use sqlx::{Pool, Row, Sqlite};

use crate::{
    api::fmp::search_symbol,
    db::write::{insert_asset, insert_ticker, insert_transaction},
    models::{Asset, AssetType, Holding, Ticker, Transaction},
};

use super::{
    calc::{calculate_position_state, calculate_transaction_gains},
    utils::{get_exchange_rate, parse_datetime, parse_decimal, parse_transaction_type},
};

#[derive(Clone, Debug, Getters)]
pub struct Portfolio {
    base_currency: String,
    connection: Pool<Sqlite>,
    transactions: Vec<Transaction>,
    holdings: Vec<Holding>,
    client: Client,
    api_key: String,
}

impl Portfolio {
    pub fn new(base_currency: String, connection: Pool<Sqlite>, api_key: String) -> Self {
        Self {
            base_currency,
            connection,
            holdings: Vec::new(),
            transactions: Vec::new(),
            client: Client::new(),
            api_key,
        }
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
                    AssetType::from_str(row.get::<&str, _>("asset_type"))?,
                    row.get::<Option<String>, _>("isin"),
                    row.get::<Option<String>, _>("sector"),
                    row.get::<Option<String>, _>("industry"),
                ),
                row.get::<String, _>("currency"),
                row.get::<String, _>("exchange"),
                Decimal::from_str(&row.get::<String, _>("last_price")).ok(),
                row.get::<Option<DateTime<Local>>, _>("last_price_updated_at"),
            );
            ticker_map.insert(symbol, (ticker, ticker_id, asset_id));
        }

        Ok(ticker_map)
    }

    async fn get_last_transaction_no(&mut self) -> Result<u32> {
        let result =
            sqlx::query_scalar::<_, Option<u32>>("SELECT MAX(transaction_no) FROM transactions")
                .fetch_one(&self.connection)
                .await?;

        Ok(result.unwrap_or(0))
    }

    pub async fn import_transactions(&mut self, path: &str) -> Result<()> {
        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file at path: {}", path))?;

        let mut ticker_map = self.get_existing_tickers().await?;
        let _last_transaction_no = self.get_last_transaction_no().await?;

        for (i, record) in reader.records().enumerate() {
            let rec = record.with_context(|| format!("Failed to read CSV record {}", i + 1))?;

            let transaction_no = rec[0].parse::<u32>()?;
            let date = parse_datetime(&rec[1])?;
            let transaction_type = parse_transaction_type(&rec[2])?;
            let symbol = rec[3].to_string();
            let quantity = parse_decimal(&rec[4], "quantity")?;
            let price = parse_decimal(&rec[5], "price")?;
            let fees = parse_decimal(&rec[6], "fees")?;
            let broker = rec[7].to_string();

            let symbol: &str = &symbol;
            let mut symbol_split = symbol.split('.');
            let standalone_symbol = symbol_split.next().unwrap_or("").to_string();
            let exchange = symbol_split.next().unwrap_or("").to_string();

            let existing_ticker = ticker_map.get(symbol);
            let (ticker, ticker_id, asset_id) = match existing_ticker {
                Some(existing_ticker) => existing_ticker,
                None => {
                    let search_result =
                        search_symbol(&standalone_symbol, &exchange, &self.client, &self.api_key)
                            .await?;
                    let ticker = search_result[0].to_ticker();
                    ticker_map.insert(standalone_symbol, (ticker.clone(), 0, 0));

                    &(ticker.clone(), 0, 0)
                }
            };

            let currency = ticker.currency();

            let exchange_rate = get_exchange_rate(
                &self.base_currency,
                &currency,
                &date,
                &self.client,
                &self.api_key,
            )
            .await?;

            let mut transaction = Transaction::new(
                transaction_no,
                date,
                transaction_type.clone(),
                ticker.clone(),
                broker.clone(),
                currency.clone(),
                exchange_rate,
                quantity,
                price,
                fees,
                None,
                None,
            );

            let (mut amounts, mut quantities): (Vec<Decimal>, Vec<Decimal>) = self
                .transactions
                .iter()
                .filter(|t| {
                    t.ticker().asset().name() == ticker.asset().name() && t.broker() == &broker
                })
                .map(|t| (t.get_amount(), t.get_quantity()))
                .unzip();

            amounts.push(transaction.get_amount());
            quantities.push(transaction.get_quantity());

            let position_state = calculate_position_state(amounts, quantities)?;
            let transaction_gains = calculate_transaction_gains(&transaction, &position_state);

            transaction.set_position_state(Some(position_state));
            transaction.set_transaction_gains(Some(transaction_gains));

            let mut new_asset_id = asset_id.clone();
            let mut new_ticker_id = ticker_id.clone();

            // TODO: Group in database transaction

            if new_asset_id == 0 {
                new_asset_id = insert_asset(transaction.ticker().asset(), &self.connection).await?;
            }

            if new_ticker_id == 0 {
                new_ticker_id = insert_ticker(ticker, &new_asset_id, &self.connection).await?;
            }

            let _ = insert_transaction(&transaction, &new_ticker_id, &self.connection).await?;

            self.transactions.push(transaction);
        }

        Ok(())
    }
}
