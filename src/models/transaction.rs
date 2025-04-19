use chrono::{DateTime, Local};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::Asset;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Transaction {
    date: DateTime<Local>,
    transaction_type: TransactionType,
    asset: Asset,
    broker: String,
    currency: String,
    quantity: Decimal,
    price: Decimal,
    fees: Decimal,
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
        }
    }
}
