use crate::models::{quote::Quote, ticker::Ticker};
use anyhow::Result;
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
}
