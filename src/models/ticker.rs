use chrono::{DateTime, Local};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Ticker {
    symbol: String,
    name: String,
    currency: String,
    exchange: String,
    last_price: Option<Decimal>,
    last_price_updated_at: Option<DateTime<Local>>,
}

impl Ticker {
    pub fn new(
        symbol: String,
        name: String,
        currency: String,
        exchange: String,
        last_price: Option<Decimal>,
        last_price_updated_at: Option<DateTime<Local>>,
    ) -> Self {
        Self {
            symbol,
            name,
            currency,
            exchange,
            last_price,
            last_price_updated_at,
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn exchange(&self) -> &str {
        &self.exchange
    }

    pub fn last_price(&self) -> Option<&Decimal> {
        self.last_price.as_ref()
    }

    pub fn last_price_updated_at(&self) -> Option<&DateTime<Local>> {
        self.last_price_updated_at.as_ref()
    }

    pub fn update_price(&mut self, price: Decimal) {
        self.last_price = Some(price);
        self.last_price_updated_at = Some(Local::now());
    }
}
