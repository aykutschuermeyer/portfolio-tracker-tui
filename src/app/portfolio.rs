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
    models::{
        Asset, AssetType, Holding, Ticker, Transaction, TransactionType, ticker::ApiProvider,
    },
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
            INNER JOIN
                cte_realized_gains_dividends rld 
                ON tnx.ticker_id = rld.ticker_id 
                AND tnx.broker = rld.broker
            INNER JOIN
                tickers tcr 
                ON tnx.ticker_id = tcr.id
            INNER JOIN
                assets ast               
                ON tcr.asset_id = ast.id 
            WHERE
                tnx.cumulative_units > 0
            "#,
        )
        .fetch_all(&self.connection)
        .await?;

        let mut holdings: Vec<Holding> = Vec::new();

        let missing_msg = |col: &str| format!("Missing '{}' column in holdings query", col);

        for row in tickers.iter() {
            let name = row
                .try_get::<String, _>("name")
                .with_context(|| missing_msg("name"))?;
            let asset_type_str = row
                .try_get::<String, _>("asset_type")
                .with_context(|| missing_msg("asset_type"))?;
            let isin = row
                .try_get::<Option<String>, _>("isin")
                .with_context(|| missing_msg("isin"))?;
            let sector = row
                .try_get::<Option<String>, _>("sector")
                .with_context(|| missing_msg("sector"))?;
            let industry = row
                .try_get::<Option<String>, _>("industry")
                .with_context(|| missing_msg("industry"))?;

            let asset = Asset::new(
                name,
                AssetType::parse_str(&asset_type_str).unwrap_or(AssetType::Stock),
                Vec::new(),
                isin,
                sector,
                industry,
            );

            let cumulative_units_f64 = row
                .try_get::<f64, _>("cumulative_units")
                .with_context(|| missing_msg("cumulative_units"))?;
            let quantity = Decimal::from_f64(cumulative_units_f64).unwrap_or(Decimal::ZERO);

            let last_price_f64 = row
                .try_get::<f64, _>("last_price")
                .with_context(|| missing_msg("last_price"))?;
            let price = Decimal::from_f64(last_price_f64).unwrap_or(Decimal::ZERO);

            let exchange_rate_f64 = row
                .try_get::<f64, _>("exchange_rate")
                .with_context(|| missing_msg("exchange_rate"))?;
            let exchange_rate = Decimal::from_f64(exchange_rate_f64).unwrap_or(dec!(1));

            let cumulative_cost_f64 = row
                .try_get::<f64, _>("cumulative_cost")
                .with_context(|| missing_msg("cumulative_cost"))?;
            let total_cost = Decimal::from_f64(cumulative_cost_f64).unwrap_or(Decimal::ZERO);

            let cost_per_share = if quantity != Decimal::ZERO {
                (total_cost / quantity).round_dp(4)
            } else {
                Decimal::ZERO
            };

            let adjusted_price = price * (dec!(1) / exchange_rate);
            let market_value = (adjusted_price * quantity).round();

            let unrealized_gain = market_value - total_cost;
            let unrealized_gain_percent = if total_cost != Decimal::ZERO {
                ((unrealized_gain / total_cost) * dec!(100)).round_dp(2)
            } else {
                Decimal::ZERO
            };

            let realized_gains_f64 = row
                .try_get::<f64, _>("realized_gains")
                .with_context(|| missing_msg("realized_gains"))?;
            let realized_gain = Decimal::from_f64(realized_gains_f64).unwrap_or(Decimal::ZERO);

            let dividends_collected_f64 = row
                .try_get::<f64, _>("dividends_collected")
                .with_context(|| missing_msg("dividends_collected"))?;
            let dividends_collected =
                Decimal::from_f64(dividends_collected_f64).unwrap_or(Decimal::ZERO);

            let total_gain = unrealized_gain + realized_gain + dividends_collected;

            let holding = Holding::new(
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
            );

            holdings.push(holding);
        }

        self.holdings.clear();
        self.holdings = holdings;

        Ok(())
    }

    async fn get_existing_tickers(&mut self) -> Result<HashMap<String, (Ticker, i64)>> {
        let tickers = sqlx::query(
            r#"
            SELECT * FROM tickers
            INNER JOIN assets on tickers.asset_id = assets.id
            "#,
        )
        .fetch_all(&self.connection)
        .await?;

        let missing_msg = |col: &str| format!("Missing '{}' column in tickers query", col);

        let mut ticker_map: HashMap<String, (Ticker, i64)> = HashMap::new();
        for row in tickers {
            let symbol: String = row
                .try_get::<String, _>("symbol")
                .with_context(|| missing_msg("symbol"))?;
            let ticker_id = row
                .try_get::<i64, _>("id")
                .with_context(|| missing_msg("id"))?;
            let name = row
                .try_get::<String, _>("name")
                .with_context(|| missing_msg("name"))?;
            let currency = row
                .try_get::<String, _>("currency")
                .with_context(|| missing_msg("currency"))?;
            let exchange = row
                .try_get("exchange")
                .with_context(|| missing_msg("exchange"))?;
            let last_price_f64 = row
                .try_get::<f64, _>("last_price")
                .with_context(|| missing_msg("last_price"))?;
            let last_price_updated_at = row
                .try_get::<Option<DateTime<Local>>, _>("last_price_updated_at")
                .with_context(|| missing_msg("last_price_updated_at"))?;
            let api_str = row
                .try_get::<&str, _>("api")
                .with_context(|| missing_msg("api"))?;

            let ticker = Ticker::new(
                symbol.clone(),
                name,
                currency,
                exchange,
                Decimal::from_f64(last_price_f64),
                last_price_updated_at,
                ApiProvider::parse_str(api_str)?,
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

        let missing_msg = |col: &str| format!("Missing '{}' column in transactions query", col);

        let mut forex_map: HashMap<i64, Decimal> = HashMap::new();
        for row in transaction_forex {
            let txn_no = row
                .try_get::<i64, _>("transaction_no")
                .with_context(|| missing_msg("transaction_no"))?;
            let exchange_rate_f64 = row
                .try_get::<f64, _>("exchange_rate")
                .with_context(|| missing_msg("exchange_rate"))?;
            let x_rate = Decimal::from_f64(exchange_rate_f64).unwrap_or(Decimal::ZERO);
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

        let headers = reader
            .headers()
            .with_context(|| format!("Failed to read CSV headers from file: {}", path))?;

        if headers.len() < 10 {
            return Err(anyhow::anyhow!(
                "Invalid CSV format: expected at least 10 columns, found {}",
                headers.len()
            ));
        }

        let mut symbols = std::collections::HashSet::new();
        for record in reader.records() {
            let rec = record?;
            if let Some(symbol) = rec.get(3) {
                symbols.insert(symbol.to_string());
            }
            if let Some(alternative_symbol) = rec.get(8) {
                if alternative_symbol.len() > 0 {
                    symbols.insert(alternative_symbol.to_string());
                }
            }
        }
        let unique_symbols: Vec<String> = symbols.into_iter().collect();

        let mut ticker_map = self.get_existing_tickers().await?;
        ticker_map = self
            .update_tickers(&unique_symbols, &mut ticker_map)
            .await?;

        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to reopen CSV file at path: {}", path))?;
        reader.headers()?;

        let mut transactions: Vec<Transaction> = Vec::new();
        let forex_map = self.get_existing_forex().await?;
        let last_transaction_no = self.get_last_transaction_no().await?;

        let mut tx = self.connection.begin().await?;

        for (i, record) in reader.records().enumerate() {
            let rec = record.with_context(|| format!("Failed to read CSV record {}", i + 1))?;

            let missing_msg =
                |col: &str, row: usize| format!("Missing '{}' column in record {}", col, row);

            let failed_to_parse_msg =
                |col: &str, row: usize| format!("Failed to parse '{}' in record {}", col, row);

            let transaction_no = rec
                .get(0)
                .with_context(|| missing_msg("transaction_no", i + 1))?
                .parse::<i64>()
                .with_context(|| failed_to_parse_msg("transaction_no", i + 1))?;

            let date = parse_datetime(rec.get(1).with_context(|| missing_msg("date", i + 1))?)
                .with_context(|| failed_to_parse_msg("date", i + 1))?;

            if last_transaction_no != 0 && (transaction_no <= last_transaction_no) {
                continue;
            }

            let transaction_type = TransactionType::parse_str(
                rec.get(2)
                    .with_context(|| missing_msg("transaction_type", i + 1))?,
            )
            .with_context(|| failed_to_parse_msg("transaction_type", i + 1))?;
            let symbol = rec
                .get(3)
                .with_context(|| missing_msg("symbol", i + 1))?
                .to_string();
            let quantity = parse_decimal(
                rec.get(4).with_context(|| missing_msg("quantity", i + 1))?,
                "quantity",
            )
            .with_context(|| failed_to_parse_msg("quantity", i + 1))?;
            let mut price = parse_decimal(
                rec.get(5).with_context(|| missing_msg("price", i + 1))?,
                "price",
            )
            .with_context(|| failed_to_parse_msg("price", i + 1))?;
            let fees = parse_decimal(
                rec.get(6).with_context(|| missing_msg("fees", i + 1))?,
                "fees",
            )
            .with_context(|| failed_to_parse_msg("fees", i + 1))?;
            let broker = rec
                .get(7)
                .with_context(|| missing_msg("broker", i + 1))?
                .to_string();
            let alternative_symbol = rec
                .get(8)
                .with_context(|| missing_msg("alternative_symbol", i + 1))?
                .to_string();
            let mut transaction_currency = rec
                .get(9)
                .with_context(|| missing_msg("transaction_currency", i + 1))?
                .to_string();

            let ticker_lookup_value = ticker_map.get(&symbol);

            let ticker_with_id = match ticker_lookup_value {
                Some(value) => value,
                None => {
                    if alternative_symbol.len() > 0 {
                        let alternative_lookup_value =
                            ticker_map.get(&alternative_symbol).with_context(|| {
                                format!(
                                    "Could not find symbols {} and {}",
                                    &symbol, &alternative_symbol
                                )
                            })?;
                        alternative_lookup_value
                    } else {
                        return Err(anyhow::anyhow!("Could not find symbol {}", &symbol));
                    }
                }
            };

            let ticker = ticker_with_id.clone().0;
            let ticker_id = ticker_with_id.clone().1;
            let currency = ticker.currency();

            if transaction_currency.len() == 0 {
                transaction_currency = ticker.currency().clone();
            }

            if &transaction_currency != currency {
                let x_rate =
                    get_exchange_rate(currency, &transaction_currency, &date, &self.client)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to get exchange rate for {} to {} in record {}",
                                currency,
                                transaction_currency,
                                i + 1
                            )
                        })?;
                price *= x_rate;
            }

            let existing_forex = forex_map.get(&transaction_no);
            let exchange_rate = match existing_forex {
                Some(existing_forex) => *existing_forex,
                None => get_exchange_rate(currency, &self.base_currency, &date, &self.client)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to get exchange rate for {} to {} in record {}",
                            currency,
                            self.base_currency,
                            i + 1
                        )
                    })?,
            };

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

            let (mut amounts, mut quantities): (Vec<Decimal>, Vec<Decimal>) = transactions
                .iter()
                .filter(|t| {
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

            let position_state =
                calculate_position_state(amounts, quantities).with_context(|| {
                    format!("Failed to calculate position state in record {}", i + 1)
                })?;
            let transaction_gains = calculate_transaction_gains(&transaction, &position_state);

            transaction.set_position_state(Some(position_state));
            transaction.set_transaction_gains(Some(transaction_gains));

            insert_transaction(&transaction, &ticker_id, &mut tx)
                .await
                .with_context(|| format!("Failed to insert transaction in record {}", i + 1))?;

            transactions.push(transaction);
        }

        tx.commit()
            .await
            .with_context(|| "Failed to commit database transaction")?;

        Ok(())
    }

    pub async fn update_tickers(
        &self,
        symbols: &Vec<String>,
        existing_tickers: &mut HashMap<String, (Ticker, i64)>,
    ) -> Result<HashMap<String, (Ticker, i64)>> {
        let mut handles = Vec::new();
        for symbol in symbols {
            let found_ticker = existing_tickers.get(symbol);
            if let Some(_ticker) = found_ticker {
                continue;
            }

            let symbol_clone = symbol.clone();
            let client = self.client.clone();
            let api_key_av = self.api_key_av.clone();
            let api_key_fmp = self.api_key_fmp.clone();
            let connection = self.connection.clone();

            let handle = tokio::spawn(async move {
                let ticker = find_ticker(&symbol_clone, &client, &api_key_fmp, &api_key_av).await?;

                let asset = Asset::new(
                    ticker.name().to_string(),
                    AssetType::Stock,
                    Vec::new(),
                    None,
                    None,
                    None,
                );

                let mut tx = connection.begin().await?;
                let new_ticker_id = insert_ticker(&ticker, &asset, &mut tx).await?;
                tx.commit().await?;

                Ok::<(String, Ticker, i64), anyhow::Error>((symbol_clone, ticker, new_ticker_id))
            });
            handles.push(handle);
        }

        for handle in handles {
            match handle.await? {
                Ok((symbol, ticker, ticker_id)) => {
                    existing_tickers.insert(symbol, (ticker, ticker_id));
                }
                Err(e) => return Err(e),
            }
        }

        Ok(existing_tickers.clone())
    }

    pub async fn update_prices(&self) -> Result<()> {
        let tickers = sqlx::query("SELECT symbol, api FROM tickers")
            .fetch_all(&self.connection)
            .await?;

        let missing_msg = |col: &str| format!("Missing '{}' column in tickers query", col);

        let mut ticker_data = Vec::new();
        for row in tickers {
            let symbol = row
                .try_get::<&str, _>("symbol")
                .with_context(|| missing_msg("symbol"))?
                .to_string();
            let api_str = row
                .try_get::<&str, _>("api")
                .with_context(|| missing_msg("api"))?;
            let api = ApiProvider::parse_str(api_str)?;
            ticker_data.push((symbol, api));
        }

        let mut handles = Vec::new();
        for (symbol, api) in ticker_data {
            let client = self.client.clone();
            let connection = self.connection.clone();
            let api_key_av = self.api_key_av.clone();
            let api_key_fmp = self.api_key_fmp.clone();

            let handle = tokio::spawn(async move {
                let price_result = match api {
                    ApiProvider::Av => {
                        let av_quote_result = av::get_quote(&symbol, &client, &api_key_av)
                            .await
                            .with_context(|| format!("Alpha Vantage ({})", &symbol))?;
                        Decimal::from_str(av_quote_result.price()).with_context(|| {
                            format!("Alpha Vantage ({}): Failed to parse price", symbol)
                        })
                    }
                    ApiProvider::Fmp => {
                        let fmp_quote_result = fmp::get_quote(&symbol, &client, &api_key_fmp)
                            .await
                            .with_context(|| format!("FMP ({})", &symbol))?;
                        Ok(*fmp_quote_result
                            .first()
                            .with_context(|| {
                                format!("FMP ({}): Failed to get first entry", symbol)
                            })?
                            .price())
                    }
                };

                match price_result {
                    Ok(price) => {
                        sqlx::query(
                            r#"
                            UPDATE tickers 
                            SET 
                                last_price = ?, 
                                last_price_updated_at = DATETIME('now'), 
                                updated_at = DATETIME('now')
                            WHERE symbol = ?
                            "#,
                        )
                        .bind(price.to_f64())
                        .bind(&symbol)
                        .execute(&connection)
                        .await?;
                        Ok(())
                    }
                    Err(e) => Err(anyhow::anyhow!(
                        "Failed to fetch price for {}: {}",
                        symbol,
                        e
                    )),
                }
            });
            handles.push(handle);
        }

        let mut errors = Vec::new();
        for handle in handles {
            match handle.await? {
                Ok(()) => {}
                Err(e) => errors.push(format!("{:#}", e)),
            }
        }

        if !errors.is_empty() {
            return Err(anyhow::anyhow!("\n{}", errors.join("\n")));
        }

        Ok(())
    }
}
