use chrono::{DateTime, Local};
use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

use super::{Asset, PositionState};

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
    position_state: Option<PositionState>,
    realized_gains: Option<Decimal>,
    dividends_collected: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionType {
    Buy,
    Sell,
    Div,
}

impl Transaction {
    pub fn get_amount_change(&self) -> Decimal {
        let amount = &self.price * &self.quantity + &self.fees;
        if self.transaction_type == TransactionType::Buy {
            return -amount.clone();
        } else {
            return amount.clone();
        }
    }

    pub fn get_quantity_change(&self) -> Decimal {
        if self.transaction_type == TransactionType::Buy {
            return self.quantity.clone();
        } else {
            return -self.quantity.clone();
        }
    }

    pub fn set_state_and_gains(
        &mut self,
        position_state: Option<PositionState>,
        realized_gains: Option<Decimal>,
        dividends_collected: Option<Decimal>,
    ) {
        self.position_state = position_state;
        self.realized_gains = realized_gains;
        self.dividends_collected = dividends_collected;
    }
}
