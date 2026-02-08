use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;

#[derive(Clone, Debug, Getters, new)]
pub struct TransactionGains {
    realized_gain: Decimal,
    dividend: Decimal,
}
