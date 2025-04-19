use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Ticker {
    symbol: String,
    name: String,
    currency: String,
    exchange: String,
}

impl Ticker {
    pub fn new(symbol: String, name: String, currency: String, exchange: String) -> Self {
        Self {
            symbol,
            name,
            currency,
            exchange,
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn exchange(&self) -> &str {
        &self.exchange
    }
}
