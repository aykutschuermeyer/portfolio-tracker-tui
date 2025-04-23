use super::Asset;
use chrono::{DateTime, Local};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

#[derive(Clone, Debug, Getters, new)]
pub struct Transaction {
    date: DateTime<Local>,
    transaction_type: TransactionType,
    asset: Asset,
    broker: String,
    currency: String,
    quantity: Decimal,
    price: Decimal,
    fees: Decimal,
    cumulative_units: Decimal,
    cumulative_cost: Decimal,
    realized_gains: Decimal,
    dividends_collected: Decimal,
}

#[derive(Clone, Debug)]
pub enum TransactionType {
    Buy,
    Sell,
    Div,
}
