use anyhow::{Context, Error, Result};
use chrono::{Local, TimeZone};
use csv::Reader;
use reqwest::Client;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{
    api::fmp,
    models::{Asset, AssetType, Position, Transaction, TransactionType},
};

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

            if rec.len() < 7 {
                return Err(Error::msg(format!(
                    "Invalid CSV format at row {}: expected at least 7 columns, found {}",
                    row_idx + 1,
                    rec.len()
                )));
            }

            let date_str = format!("{} 00:00:00", &rec[0]);
            let naive = chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")
                .with_context(|| {
                    format!("Failed to parse date '{}' at row {}", &rec[0], row_idx + 1)
                })?;
            let date = Local.from_utc_datetime(&naive);

            let transaction_type = match &rec[1] {
                "Buy" => TransactionType::Buy,
                "Sell" => TransactionType::Sell,
                "Div" => TransactionType::Div,
                other => {
                    eprintln!(
                        "Warning: Skipping unknown transaction type '{}' at row {}",
                        other,
                        row_idx + 1
                    );
                    continue;
                }
            };

            let symbol = rec[2].to_string();

            let quantity = rec[3].parse::<Decimal>().with_context(|| {
                format!(
                    "Failed to parse quantity '{}' at row {}",
                    &rec[3],
                    row_idx + 1
                )
            })?;

            let price = rec[4].parse::<Decimal>().with_context(|| {
                format!("Failed to parse price '{}' at row {}", &rec[4], row_idx + 1)
            })?;

            let fees = rec[5].parse::<Decimal>().with_context(|| {
                format!("Failed to parse fees '{}' at row {}", &rec[5], row_idx + 1)
            })?;

            let broker = rec[6].to_string();

            let mut symbol_split = symbol.split('.');
            let standalone_symbol = symbol_split.next().unwrap_or("");
            let exchange = symbol_split.next().unwrap_or("");

            let search_results = match fmp::search_symbol(
                standalone_symbol,
                exchange,
                &self.client,
                &self.api_key,
            )
            .await
            {
                Ok(results) => results,
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to find ticker for symbol '{}' on exchange '{}' at row {}: {}",
                        standalone_symbol,
                        exchange,
                        row_idx + 1,
                        err
                    );
                    continue;
                }
            };

            let fmp_symbol = &search_results[0];
            let currency = fmp_symbol.currency().to_string();
            let mut ticker = crate::models::Ticker::new(
                fmp_symbol.symbol().to_string(),
                fmp_symbol.name().to_string(),
                currency.clone(),
                fmp_symbol.exchange().to_string(),
                None,
                None,
            );

            let quotes = match fmp::get_quote(&ticker.symbol(), &self.client, &self.api_key).await {
                Ok(quotes) => quotes,
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to get quote for ticker '{}' at row {}: {}",
                        ticker.symbol(),
                        row_idx + 1,
                        err
                    );
                    continue;
                }
            };

            if quotes.is_empty() {
                eprintln!(
                    "Warning: No quotes found for ticker '{}' at row {}",
                    ticker.symbol(),
                    row_idx + 1
                );
                continue;
            }

            let price_decimal = *quotes[0].price();
            ticker.update_price(price_decimal);

            let asset = Asset::new(
                ticker.name().to_string(),
                AssetType::Stock,
                Vec::from([ticker]),
                None,
                None,
                None,
            );

            let ticker_symbol = asset.tickers()[0].symbol().to_string();

            let (
                mut cumulative_units,
                mut cumulative_cost,
                mut realized_gains,
                mut dividends_collected,
            ) = position_state.get(&ticker_symbol).cloned().unwrap_or((
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
                ticker_symbol,
                (
                    cumulative_units,
                    cumulative_cost,
                    realized_gains,
                    dividends_collected,
                ),
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
