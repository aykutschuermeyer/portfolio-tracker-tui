use std::path::Path;

use crate::models::{Asset, AssetType, Quote, Ticker, Transaction, TransactionType};
use anyhow::Result;
use chrono::{Local, NaiveDate, TimeZone};
use csv::Reader;
use reqwest::Client;
use rust_decimal::Decimal;

pub struct PortfolioTrackerService {
    client: Client,
    api_key: String,
}

impl PortfolioTrackerService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_quote(&self, ticker: &Ticker) -> Result<Quote> {
        let url = format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            ticker.symbol(),
            self.api_key
        );

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(global_quote) = data["Global Quote"].as_object() {
            let quote = Quote::new(
                global_quote["01. symbol"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                global_quote["02. open"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<Decimal>()?,
                global_quote["03. high"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<Decimal>()?,
                global_quote["04. low"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<Decimal>()?,
                global_quote["05. price"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<Decimal>()?,
                global_quote["06. volume"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<i64>()?,
                global_quote["07. latest trading day"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                global_quote["08. previous close"]
                    .as_str()
                    .unwrap_or("")
                    .parse::<Decimal>()?,
                global_quote["09. change"]
                    .as_str()
                    .unwrap_or("")
                    .parse::<Decimal>()?,
                global_quote["10. change percent"]
                    .as_str()
                    .unwrap_or("")
                    .parse::<Decimal>()?,
            );
            Ok(quote)
        } else {
            Err(anyhow::anyhow!("No quote data found"))
        }
    }

    pub fn read_transactions<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Transaction>> {
        let mut reader = Reader::from_path(path)?;
        let mut transactions = Vec::new();

        for result in reader.records() {
            let record = result?;
            let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")?;
            let date = Local
                .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
                .single()
                .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;

            let transaction_type = match &record[1] {
                "Buy" => TransactionType::Buy,
                "Sell" => TransactionType::Sell,
                "Div" => TransactionType::Div,
                _ => return Err(anyhow::anyhow!("Invalid transaction type")),
            };

            let symbol = record[2].to_string();
            let quantity = record[3].parse::<Decimal>()?;
            let price = record[4].parse::<Decimal>()?;
            let fees = record[5].parse::<Decimal>()?;

            // Create a temporary Ticker for the transaction
            let ticker = Ticker::new(symbol.clone(), "USD".to_string(), "".to_string());
            let asset = Asset::new(symbol, AssetType::Stock, ticker, None, None, None, None);

            let transaction = Transaction::new(
                date,
                transaction_type,
                asset,
                "".to_string(),    // broker
                "USD".to_string(), // currency
                quantity,
                price,
                fees,
            );

            transactions.push(transaction);
        }

        Ok(transactions)
    }
}
