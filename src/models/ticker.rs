use anyhow::Result;
use rust_decimal::Decimal;

pub struct Ticker {
    symbol: String,
    currency: String,
    exchange: String,
}

pub struct GlobalQuote {
    symbol: String,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    price: Decimal,
    volume: i64,
    date: String,
    previous_close: Decimal,
    change: Decimal,
    change_percent: Decimal,
}

impl Ticker {
    pub fn new(symbol: String, currency: String, exchange: String) -> Self {
        Self {
            symbol,
            currency,
            exchange,
        }
    }

    pub async fn get_quote(&self) -> Result<GlobalQuote> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
            .expect("Missing environment variable ALPHA_VANTAGE_API_KEY");

        let url = format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            self.symbol, api_key
        );

        let response = client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(global_quote) = data["Global Quote"].as_object() {
            let quote = GlobalQuote {
                symbol: global_quote["01. symbol"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                open: global_quote["02. open"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<Decimal>()?,
                high: global_quote["03. high"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<Decimal>()?,
                low: global_quote["04. low"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<Decimal>()?,
                price: global_quote["05. price"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<Decimal>()?,
                volume: global_quote["06. volume"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<i64>()?,
                date: global_quote["07. latest trading day"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                previous_close: global_quote["08. previous close"]
                    .as_str()
                    .unwrap_or("")
                    .parse::<Decimal>()?,
                change: global_quote["09. change"]
                    .as_str()
                    .unwrap_or("")
                    .parse::<Decimal>()?,
                change_percent: global_quote["10. change percent"]
                    .as_str()
                    .unwrap_or("")
                    .parse::<Decimal>()?,
            };
            Ok(quote)
        } else {
            Err(anyhow::anyhow!("No quote data found"))
        }
    }
}
