use chrono::{DateTime, Local};
use derive_getters::Getters;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
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

    pub fn update_price(&mut self, price: Decimal) {
        self.last_price = Some(price);
        self.last_price_updated_at = Some(Local::now());
    }
}
