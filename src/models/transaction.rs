use chrono::{DateTime, Local};
use derive_getters::Getters;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::Asset;

#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TransactionType {
    Buy,
    Sell,
    Div,
}

impl Transaction {
    pub fn new(
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
    ) -> Self {
        Self {
            date,
            transaction_type,
            asset,
            broker,
            currency,
            quantity,
            price,
            fees,
            cumulative_units,
            cumulative_cost,
            realized_gains,
            dividends_collected,
        }
    }
}
