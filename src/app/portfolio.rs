use crate::{
    api::FmpApi,
    models::{Asset, AssetType, Position, Transaction, TransactionType},
};
use anyhow::{Context, Error, Result};
use chrono::{Local, TimeZone};
use csv::Reader;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Portfolio {
    transactions: Vec<Transaction>,
    positions: Vec<Position>,
    api: FmpApi,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            transactions: Vec::new(),
            api: FmpApi::new(),
        }
    }

    pub async fn import_transactions(&mut self, path: &str) -> Result<()> {
        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file at path: {}", path))?;

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

            let mut symbol_split = symbol.split(".");
            let standalone_symbol = symbol_split.next().unwrap_or("");
            let exchange = symbol_split.next().unwrap_or("");

            let mut ticker = match self.api.search(standalone_symbol, exchange).await {
                Ok(ticker) => ticker,
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

            let currency = ticker.currency().to_string();

            let quotes = match self.api.get_quote(&ticker).await {
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

            ticker.update_price(*quotes[0].price());

            let asset = Asset::new(
                ticker.name().to_string(),
                AssetType::Stock,
                Vec::from([ticker]),
                None,
                None,
                None,
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
            );

            self.transactions.push(transaction);
        }

        Ok(())
    }

    pub fn positions(&self) -> &Vec<Position> {
        &self.positions
    }

    pub fn calculate_positions(&mut self) -> Result<()> {
        if self.transactions.is_empty() {
            return Err(Error::msg("No transactions to calculate positions from"));
        }

        let mut sorted_transactions = self.transactions.clone();
        sorted_transactions.sort_by(|a, b| a.date().cmp(&b.date()));

        let mut asset_transactions: HashMap<String, Vec<Transaction>> = HashMap::new();

        for transaction in sorted_transactions {
            if transaction.asset().tickers().is_empty() {
                continue;
            }

            let ticker = transaction.asset().tickers()[0].symbol().to_string();
            asset_transactions
                .entry(ticker)
                .or_insert_with(Vec::new)
                .push(transaction);
        }

        self.positions.clear();

        for (ticker_symbol, transactions) in asset_transactions {
            let mut fifo_lots: Vec<(Decimal, Decimal, Decimal)> = Vec::new();
            let mut realized_gain = Decimal::ZERO;

            for transaction in &transactions {
                match transaction.transaction_type() {
                    TransactionType::Buy => {
                        let quantity = transaction.quantity().clone();
                        let price = transaction.price().clone();
                        let fees = transaction.fees().clone();
                        fifo_lots.push((quantity, price, fees));
                    }
                    TransactionType::Sell => {
                        let mut remaining_sell_quantity = transaction.quantity().clone();
                        let sell_price = transaction.price();

                        let total_buy_quantity: Decimal =
                            fifo_lots.iter().map(|(qty, _, _)| qty).sum();
                        if total_buy_quantity < remaining_sell_quantity {
                            eprintln!(
                                "Warning: Attempting to sell more shares than owned for {}: sell quantity {}, owned quantity {}",
                                ticker_symbol, remaining_sell_quantity, total_buy_quantity
                            );
                        }

                        while remaining_sell_quantity > Decimal::ZERO && !fifo_lots.is_empty() {
                            let (lot_quantity, lot_price, lot_fees) = fifo_lots[0];

                            let sell_from_lot = if remaining_sell_quantity >= lot_quantity {
                                lot_quantity
                            } else {
                                remaining_sell_quantity
                            };

                            let cost_basis = lot_price * sell_from_lot;
                            let proportional_fees = if lot_quantity > Decimal::ZERO {
                                lot_fees * (sell_from_lot / lot_quantity)
                            } else {
                                Decimal::ZERO
                            };
                            let total_cost = cost_basis + proportional_fees;

                            let proceeds = sell_price * sell_from_lot;

                            let gain_loss = proceeds - total_cost;
                            realized_gain += gain_loss;

                            remaining_sell_quantity -= sell_from_lot;

                            if sell_from_lot >= lot_quantity {
                                fifo_lots.remove(0);
                            } else {
                                fifo_lots[0].0 -= sell_from_lot;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if fifo_lots.is_empty() {
                continue;
            }

            let mut total_quantity = Decimal::ZERO;
            let mut total_cost = Decimal::ZERO;

            for (quantity, price, fees) in &fifo_lots {
                total_quantity += *quantity;
                total_cost += (*price * *quantity) + *fees;
            }

            let cost_per_share = if total_quantity > Decimal::ZERO {
                total_cost / total_quantity
            } else {
                Decimal::ZERO
            };

            if let Some(last_transaction) = transactions.last() {
                let asset = last_transaction.asset().clone();
                if asset.tickers().is_empty() {
                    eprintln!(
                        "Warning: Asset {} has no tickers, skipping position calculation",
                        asset.name()
                    );
                    continue;
                }
                let current_price = asset.tickers()[0].last_price().unwrap_or(Decimal::ZERO);
                let market_value = current_price * total_quantity;
                let unrealized_gain = market_value - total_cost;
                let unrealized_gain_percent = if total_cost > Decimal::ZERO {
                    (unrealized_gain / total_cost) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };

                let total_gain = realized_gain + unrealized_gain;

                let position = Position::new(
                    asset.clone(),
                    total_quantity,
                    current_price,
                    market_value,
                    cost_per_share,
                    total_cost,
                    unrealized_gain,
                    unrealized_gain_percent,
                    realized_gain,
                    total_gain,
                );

                self.positions.push(position);
            }
        }

        Ok(())
    }
}
