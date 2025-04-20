use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Quote {
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

impl Quote {
    pub fn new(
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
    ) -> Self {
        Self {
            symbol,
            open,
            high,
            low,
            price,
            volume,
            date,
            previous_close,
            change,
            change_percent,
        }
    }

    pub fn price(&self) -> &Decimal {
        &self.price
    }
}
