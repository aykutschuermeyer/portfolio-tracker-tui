use rust_decimal::Decimal;

pub struct Ticker {
    symbol: String,
    currency: String,
    exchange: String,
}

impl Ticker {
    pub fn new(symbol: String, currency: String, exchange: String) -> Self {
        Self {
            symbol,
            currency,
            exchange,
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn exchange(&self) -> &str {
        &self.exchange
    }
}
