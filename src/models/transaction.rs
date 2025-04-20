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

    pub fn date(&self) -> &DateTime<Local> {
        &self.date
    }

    pub fn transaction_type(&self) -> TransactionType {
        self.transaction_type
    }

    pub fn asset(&self) -> &Asset {
        &self.asset
    }

    pub fn broker(&self) -> &str {
        &self.broker
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn quantity(&self) -> &Decimal {
        &self.quantity
    }

    pub fn price(&self) -> &Decimal {
        &self.price
    }

    pub fn fees(&self) -> &Decimal {
        &self.fees
    }
}
