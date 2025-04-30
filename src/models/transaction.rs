use chrono::{DateTime, Local};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

use super::{Asset, PositionState, TransactionGains};

#[derive(Clone, Debug, Getters, new)]
pub struct Transaction {
    date: DateTime<Local>,
    transaction_type: TransactionType,
    asset: Asset,
    broker: String,
    currency: String,
    exchange_rate: Decimal,
    quantity: Decimal,
    price: Decimal,
    fees: Decimal,
    position_state: Option<PositionState>,
    transaction_gains: Option<TransactionGains>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionType {
    Buy,
    Sell,
    Div,
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
