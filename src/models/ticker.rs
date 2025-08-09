use anyhow::Result;
use chrono::{DateTime, Local};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

#[derive(Clone, Debug, Getters, new)]
pub struct Ticker {
    symbol: String,
    name: String,
    currency: String,
    exchange: Option<String>,
    last_price: Option<Decimal>,
    last_price_updated_at: Option<DateTime<Local>>,
    last_api: ApiProvider,
}

impl Ticker {
    pub fn update_price(&mut self, price: Decimal) {
        self.last_price = Some(price);
        self.last_price_updated_at = Some(Local::now());
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ApiProvider {
    Av,
    Fmp,
}

impl ApiProvider {
    pub fn parse_str(s: &str) -> Result<ApiProvider> {
        match s {
            "Alpha Vantage" => Ok(ApiProvider::Av),
            "Financial Modeling Prep" => Ok(ApiProvider::Fmp),
            _ => Err(anyhow::anyhow!("Unknown API provider")),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            ApiProvider::Av => "Alpha Vantage",
            ApiProvider::Fmp => "Financial Modeling Prep",
        }
    }
}
