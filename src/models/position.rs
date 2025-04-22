use derive_getters::Getters;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::Asset;

#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Position {
    asset: Asset,
    quantity: Decimal,
    price: Decimal,
    market_value: Decimal,
    cost_per_share: Decimal,
    total_cost: Decimal,
    unrealized_gain: Decimal,
    unrealized_gain_percent: Decimal,
    realized_gain: Decimal,
    dividends_collected: Decimal,
    total_gain: Decimal,
}

impl Position {
    pub fn new(
        asset: Asset,
        quantity: Decimal,
        price: Decimal,
        market_value: Decimal,
        cost_per_share: Decimal,
        total_cost: Decimal,
        unrealized_gain: Decimal,
        unrealized_gain_percent: Decimal,
        realized_gain: Decimal,
        dividends_collected: Decimal,
        total_gain: Decimal,
    ) -> Self {
        Self {
            asset,
            quantity,
            price,
            market_value,
            cost_per_share,
            total_cost,
            unrealized_gain,
            unrealized_gain_percent,
            realized_gain,
            dividends_collected,
            total_gain,
        }
    }
}
