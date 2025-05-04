use anyhow::Result;
use chrono::{DateTime, Local};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

use super::{PositionState, Ticker, TransactionGains};

#[derive(Clone, Debug, Getters, new)]
pub struct Transaction {
    transaction_no: i64,
    date: DateTime<Local>,
    transaction_type: TransactionType,
    ticker: Ticker,
    broker: String,
    currency: String,
    exchange_rate: Decimal,
    quantity: Decimal,
    price: Decimal,
    fees: Decimal,
    position_state: Option<PositionState>,
    transaction_gains: Option<TransactionGains>,
}

impl Transaction {
    pub fn get_amount(&self) -> Decimal {
        let amount = &self.price * &self.exchange_rate * &self.quantity + &self.fees;
        if self.transaction_type == TransactionType::Buy {
            return -amount.clone();
        } else {
            return amount.clone();
        }
    }

    pub fn get_quantity(&self) -> Decimal {
        if self.transaction_type == TransactionType::Buy {
            return self.quantity.clone();
        } else {
            return -self.quantity.clone();
        }
    }

    pub fn set_position_state(&mut self, position_state: Option<PositionState>) {
        self.position_state = position_state;
    }

    pub fn set_transaction_gains(&mut self, transaction_gains: Option<TransactionGains>) {
        self.transaction_gains = transaction_gains;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionType {
    Buy,
    Sell,
    Div,
}

impl TransactionType {
    pub fn from_str(s: &str) -> Result<TransactionType> {
        match s {
            "Buy" => Ok(TransactionType::Buy),
            "Sell" => Ok(TransactionType::Sell),
            "Div" => Ok(TransactionType::Div),
            _ => Err(anyhow::anyhow!("Unknown transaction type")),
        }
    }
    pub fn to_str(&self) -> &str {
        match self {
            TransactionType::Buy => "Buy",
            TransactionType::Sell => "Sell",
            TransactionType::Div => "Div",
        }
    }
}
