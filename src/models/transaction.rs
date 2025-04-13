use chrono::{DateTime, Local};
use rust_decimal::Decimal;

use super::Asset;

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
