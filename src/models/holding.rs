use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

use super::Asset;

#[derive(Clone, Debug, Getters, new)]
pub struct Holding {
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
