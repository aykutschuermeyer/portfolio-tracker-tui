use chrono::{DateTime, Local};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

#[derive(Clone, Debug, Getters, new)]
pub struct Ticker {
    symbol: String,
    name: String,
    currency: String,
    exchange: String,
    last_price: Option<Decimal>,
    last_price_updated_at: Option<DateTime<Local>>,
}

impl Ticker {
    pub fn update_price(&mut self, price: Decimal) {
        self.last_price = Some(price);
        self.last_price_updated_at = Some(Local::now());
    }
}
