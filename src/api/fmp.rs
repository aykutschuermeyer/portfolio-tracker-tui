use crate::models::{Quote, Ticker};
use anyhow::{Error, Result};
use chrono::{TimeZone, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use serde_json::Value;

const BASE_URL: &str = "https://financialmodelingprep.com/stable";

#[derive(Clone, Debug)]
pub struct FmpApi {
    client: Client,
    api_key: String,
}

impl Default for FmpApi {
    fn default() -> Self {
        Self::new()
    }
}

impl FmpApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: std::env::var("FMP_API_KEY").expect("Missing FMP_API_KEY in enviroment"),
        }
    }

    async fn make_request(&self, endpoint: &str) -> Result<Value> {
        let url = format!("{}/{}&apikey={}", BASE_URL, endpoint, self.api_key);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(Error::msg(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        let text = response.text().await?;

        if let Ok(data) = serde_json::from_str::<Vec<Value>>(&text) {
            if !data.is_empty() {
                return Ok(Value::Array(data));
            }
            return Err(Error::msg("Empty API response"));
        }

        Err(Error::msg(format!("Unexpected API response: {}", text)))
    }

    pub async fn search(&self, symbol: &str, exchange: &str) -> Result<Ticker> {
        let endpoint = format!(
            "search-symbol?query={}&exchange={}&limit=1",
            symbol, exchange
        );
        let result = self.make_request(&endpoint).await?;

        println!("{}", result);

        if let Some(data) = result.as_array() {
            if let Some(first_entry) = data.get(0) {
                return Ok(Ticker::new(
                    first_entry["symbol"].as_str().unwrap_or("").to_string(),
                    first_entry["name"].as_str().unwrap_or("").to_string(),
                    first_entry["currency"].as_str().unwrap_or("").to_string(),
                    first_entry["exchange"].as_str().unwrap_or("").to_string(),
                ));
            }
        }

        Err(Error::msg(format!(
            "No ticker found for symbol {symbol} on exchange {exchange}"
        )))
    }

    pub async fn get_quote(&self, ticker: &Ticker) -> Result<Quote> {
        let endpoint = format!("quote/{}?", ticker.symbol());
        let result = self.make_request(&endpoint).await?;

        if let Some(data) = result.as_array() {
            if let Some(quote_data) = data.first() {
                // Helper to safely extract values
                let get_decimal = |field: &str| -> Result<Decimal> {
                    quote_data[field]
                        .as_str()
                        .unwrap_or("0")
                        .parse::<Decimal>()
                        .map_err(|e| Error::msg(format!("Failed to parse {}: {}", field, e)))
                };

                let quote = Quote::new(
                    quote_data["symbol"].as_str().unwrap_or("").to_string(),
                    get_decimal("open")?,
                    get_decimal("dayHigh")?,
                    get_decimal("dayLow")?,
                    get_decimal("price")?,
                    quote_data["volume"]
                        .as_str()
                        .unwrap_or("0")
                        .parse::<i64>()?,
                    Utc.timestamp_opt(quote_data["timestamp"].as_i64().unwrap_or(0), 0)
                        .single()
                        .map(|dt| dt.to_string())
                        .unwrap_or_else(|| "Invalid timestamp".to_string()),
                    get_decimal("previousClose")?,
                    get_decimal("change")?,
                    get_decimal("changePercentage")?,
                );
                return Ok(quote);
            }
        }

        Err(Error::msg(format!(
            "No quote data found for ticker {}",
            ticker.symbol()
        )))
    }
}
