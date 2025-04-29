use std::collections::VecDeque;

use anyhow::Result;
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::models::PositionState;

pub fn fifo(amounts: Vec<Decimal>, quantities: Vec<Decimal>) -> Result<PositionState> {
    let mut queue = VecDeque::new();
    let mut cost_of_goods_sold = Decimal::ZERO;
    let mut remaining_quantity = Decimal::ZERO;

    for i in 0..amounts.len() {
        cost_of_goods_sold = Decimal::ZERO;
        let amount = amounts[i];
        let quantity = quantities[i];
        let unit_cost = amount / quantity;
        remaining_quantity += quantity;

        let quantity_abs = quantity
            .abs()
            .floor()
            .to_i32()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert quantity to i32"))?;

        if amount < Decimal::ZERO {
            for _ in 0..quantity_abs {
                queue.push_back(unit_cost);
            }
        }

        if amount > Decimal::ZERO {
            for _ in 0..quantity_abs {
                cost_of_goods_sold += queue[0];
                queue.pop_front();
            }

            // Correct for edge case with decimal units
            if remaining_quantity.round_dp(4) == Decimal::ZERO {
                while queue.len() > 0 {
                    queue.pop_front();
                }
            }
        }
    }

    let cost_of_remaining = queue
        .iter()
        .fold(Decimal::ZERO, |sum, &unit_cost| sum + unit_cost);

    Ok(PositionState::new(
        remaining_quantity.abs().round_dp(4),
        cost_of_remaining.abs(),
        cost_of_goods_sold.abs(),
    ))
}
