use std::collections::HashMap;

use anyhow::{Context, Error, Result};
use chrono::Local;
use csv::Reader;
use reqwest::Client;
use rust_decimal::Decimal;

use crate::{
    api::fmp::{get_quote, search_symbol},
    models::{Position, Ticker, Transaction},
};

use super::utils::parse_transaction;

#[derive(Clone, Debug, Default)]
pub struct Portfolio {
    transactions: Vec<Transaction>,
    positions: Vec<Position>,
    client: Client,
    api_key: String,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            transactions: Vec::new(),
            client: Client::new(),
            api_key: std::env::var("FMP_API_KEY").unwrap_or_else(|_| "".to_string()),
        }
    }

    pub fn positions(&self) -> &Vec<Position> {
        &self.positions
    }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub async fn import_transactions(&mut self, path: &str) -> Result<()> {
        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file at path: {}", path))?;

        let mut position_state: HashMap<String, (Decimal, Decimal, Decimal, Decimal)> =
            HashMap::new();

        for (row_idx, record) in reader.records().enumerate() {
            let rec = record
                .with_context(|| format!("Failed to read CSV record at row {}", row_idx + 1))?;

            let transaction_result = match parse_transaction(&rec, row_idx) {
                Ok(result) => result,
                Err(err) => {
                    eprintln!("Warning: {}", err);
                    continue;
                }
            };

            let (date, transaction_type, symbol, quantity, price, fees, broker) =
                transaction_result;

            let (standalone_symbol, exchange) = {
                let symbol: &str = &symbol;
                let mut symbol_split = symbol.split('.');
                let standalone_symbol = symbol_split.next().unwrap_or("").to_string();
                let exchange = symbol_split.next().unwrap_or("").to_string();

                (standalone_symbol, exchange)
            };

            let ticker_result = match {
                let symbol: &str = &standalone_symbol;
                let exchange: &str = &exchange;
                let client: &Client = &self.client;
                let api_key: &str = &self.api_key;
                let row_idx = row_idx;
                async move {
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
            }
            .await
            {
                Ok(result) => result,
                Err(err) => {
                    eprintln!("Warning: {}", err);
                    continue;
                }
            };

            let (ticker, currency) = ticker_result;

            let asset = crate::app::utils::create_asset(ticker);
            let ticker_symbol = asset.tickers()[0].symbol().to_string();

            let (cumulative_units, cumulative_cost, realized_gains, dividends_collected) =
                crate::app::utils::update_position_state(
                    &mut position_state,
                    &ticker_symbol,
                    &transaction_type,
                    quantity,
                    price,
                    fees,
                );

            let transaction = Transaction::new(
                date,
                transaction_type,
                asset,
                broker,
                currency,
                quantity,
                price,
                fees,
                cumulative_units,
                cumulative_cost,
                realized_gains,
                dividends_collected,
            );

            self.transactions.push(transaction);
        }

        self.positions.clear();
        for (ticker_symbol, (quantity, total_cost, realized_gain, dividends_collected)) in
            position_state
        {
            if quantity <= Decimal::ZERO {
                continue;
            }

            if let Some(last_transaction) = self.transactions.iter().rev().find(|t| {
                !t.asset().tickers().is_empty()
                    && t.asset().tickers()[0].symbol().to_string() == ticker_symbol
            }) {
                let asset = last_transaction.asset().clone();
                if let Some(current_price) = asset.tickers()[0].last_price().clone() {
                    let cost_per_share = if quantity > Decimal::ZERO {
                        total_cost / quantity
                    } else {
                        Decimal::ZERO
                    };

                    let market_value = current_price * quantity;
                    let unrealized_gain = market_value - total_cost;
                    let unrealized_gain_percent = if total_cost > Decimal::ZERO {
                        (unrealized_gain / total_cost) * Decimal::from(100)
                    } else {
                        Decimal::ZERO
                    };

                    let total_gain = realized_gain + unrealized_gain;

                    let position = Position::new(
                        asset.clone(),
                        quantity,
                        current_price,
                        market_value,
                        cost_per_share,
                        total_cost,
                        unrealized_gain,
                        unrealized_gain_percent,
                        realized_gain,
                        dividends_collected,
                        total_gain,
                    );

                    self.positions.push(position);
                }
            }
        }

        Ok(())
    }
}
