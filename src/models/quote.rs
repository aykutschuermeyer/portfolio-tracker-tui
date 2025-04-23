use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

#[derive(Clone, Debug, Getters, new)]
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
