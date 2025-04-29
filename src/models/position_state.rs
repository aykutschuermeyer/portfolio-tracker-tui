use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

#[derive(Clone, Debug, Getters, new)]
pub struct PositionState {
    cumulative_units: Decimal,
    cumulative_cost: Decimal,
    cost_of_units_sold: Decimal,
}
