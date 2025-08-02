use std::collections::VecDeque;

use anyhow::Result;
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::models::{PositionState, Transaction, TransactionGains, TransactionType};

pub fn calculate_position_state(
    amounts: Vec<Decimal>,
    quantities: Vec<Decimal>,
) -> Result<PositionState> {
    let mut queue = VecDeque::new();
    let mut cost_of_units_sold = Decimal::ZERO;
    let mut cumulative_units = Decimal::ZERO;

    for i in 0..amounts.len() {
        cost_of_units_sold = Decimal::ZERO;
        let amount = amounts[i];
        let quantity = quantities[i];
        let unit_cost = amount / quantity;
        cumulative_units += quantity;

        let quantity_abs = quantity
            .abs()
            .floor()
            .to_i64()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert quantity to i64"))?;

        if amount < Decimal::ZERO {
            for _ in 0..quantity_abs {
                queue.push_back(unit_cost);
            }
        }

        if amount > Decimal::ZERO {
            for _ in 0..quantity_abs {
                cost_of_units_sold += queue[0];
                queue.pop_front();
            }

            // Correct for edge case with decimal units
            if cumulative_units.round_dp(4) == Decimal::ZERO {
                while !queue.is_empty() {
                    queue.pop_front();
                }
            }
        }
    }

    let cumulative_cost = queue
        .iter()
        .fold(Decimal::ZERO, |sum, &unit_cost| sum + unit_cost);

    Ok(PositionState::new(
        cumulative_units.abs().round_dp(4),
        cumulative_cost.abs(),
        cost_of_units_sold.abs(),
    ))
}

pub fn calculate_transaction_gains(
    transaction: &Transaction,
    position_state: &PositionState,
) -> TransactionGains {
    let mut realized_gains = Decimal::ZERO;
    let mut dividends_collected = Decimal::ZERO;

    if transaction.transaction_type() == &TransactionType::Sell {
        realized_gains = transaction.get_amount().abs() - position_state.cost_of_units_sold();
    }

    if transaction.transaction_type() == &TransactionType::Div {
        dividends_collected = transaction.get_amount();
    }

    TransactionGains::new(realized_gains, dividends_collected)
}
