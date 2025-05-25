use std::collections::HashMap;

use derive_getters::Getters;
use derive_new::new;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Getters, new)]
#[serde(rename_all = "camelCase")]
pub struct FrankForexDto {
    amount: Decimal,
    base: String,
    date: String,
    rates: HashMap<String, Decimal>,
}
