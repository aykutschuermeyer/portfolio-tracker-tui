use anyhow::{Error, Result};

use crate::{
    api::fmp::FmpApi,
    models::{Asset, AssetType, Position, Transaction, TransactionType},
};
use chrono::{Local, TimeZone};
use csv::Reader;
use rust_decimal::Decimal;

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

    pub async fn import_transactions(&mut self, path: &str) -> Result<bool> {
        let mut reader = Reader::from_path(path)?;
        for record in reader.records() {
            let rec = record?;
            let date_str = format!("{} 00:00:00", &rec[0]);
            let naive = chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")?;
            let date = Local.from_utc_datetime(&naive);

            let transaction_type = match &rec[1] {
                "Buy" => TransactionType::Buy,
                "Sell" => TransactionType::Sell,
                _ => {
                    return Err(Error::msg("Invalid transaction type"));
                }
            };

            let symbol = rec[2].to_string();
            let quantity = rec[3].parse::<Decimal>()?;
            let price = rec[4].parse::<Decimal>()?;
            let fees = rec[5].parse::<Decimal>()?;
            let broker = rec[6].to_string();

            let mut symbol_split = symbol.split(".");
            let standalone_symbol = symbol_split.next().unwrap_or("");
            let exchange = symbol_split.next().unwrap_or("");

            let ticker = self.api.search(standalone_symbol, exchange).await?;

            let currency = ticker.currency().to_string();

            let asset = Asset::new(
                String::from(""),
                AssetType::Stock,
                ticker,
                None,
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

        Ok(true)
    }

    pub fn calculate_positions(&self) {}
}
