use anyhow::{Context, Result};
use csv::Reader;
use derive_getters::Getters;
use reqwest::Client;
use rust_decimal::Decimal;

use crate::{
    api::fmp::search_symbol,
    models::{Holding, Transaction},
};

use super::{
    calc::{calculate_gains, fifo},
    utils::{
        create_asset, get_exchange_rate, parse_datetime, parse_decimal, parse_transaction_type,
    },
};

#[derive(Clone, Debug, Default, Getters)]
pub struct Portfolio {
    base_currency: String,
    transactions: Vec<Transaction>,
    holdings: Vec<Holding>,
    client: Client,
    api_key: String,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            base_currency: String::from("EUR"),
            holdings: Vec::new(),
            transactions: Vec::new(),
            client: Client::new(),
            api_key: std::env::var("FMP_API_KEY").unwrap_or_else(|_| "".to_string()),
        }
    }

    pub async fn import_transactions(&mut self, path: &str) -> Result<()> {
        let mut reader = Reader::from_path(path)
            .with_context(|| format!("Failed to open CSV file at path: {}", path))?;

        for (i, record) in reader.records().enumerate() {
            let rec = record.with_context(|| format!("Failed to read CSV record {}", i + 1))?;

            let date = parse_datetime(&rec[0], i)?;
            let transaction_type = parse_transaction_type(&rec[1], i)?;
            let symbol = rec[2].to_string();
            let quantity = parse_decimal(&rec[3], "quantity", i)?;
            let price = parse_decimal(&rec[4], "price", i)?;
            let fees = parse_decimal(&rec[5], "fees", i)?;
            let broker = rec[6].to_string();

            let symbol: &str = &symbol;
            let mut symbol_split = symbol.split('.');
            let standalone_symbol = symbol_split.next().unwrap_or("").to_string();
            let exchange = symbol_split.next().unwrap_or("").to_string();

            let search_symbol_result =
                search_symbol(&standalone_symbol, &exchange, &self.client, &self.api_key).await?;

            let ticker = search_symbol_result[0].to_ticker();
            let currency = ticker.currency().clone();

            let exchange_rate = get_exchange_rate(
                &self.base_currency,
                &currency,
                &date,
                &self.client,
                &self.api_key,
            )
            .await?;

            let asset = create_asset(ticker);

            let mut transaction = Transaction::new(
                date,
                transaction_type.clone(),
                asset.clone(),
                broker.clone(),
                currency,
                exchange_rate,
                quantity,
                price,
                fees,
                None,
                None,
            );

            let (mut amounts, mut quantities): (Vec<Decimal>, Vec<Decimal>) = self
                .transactions
                .iter()
                .filter(|t| t.asset().name() == asset.name() && t.broker() == &broker)
                .map(|t| (t.get_amount(), t.get_quantity()))
                .unzip();

            amounts.push(transaction.get_amount());
            quantities.push(transaction.get_quantity());

            let position_state = fifo(amounts, quantities)?;
            let transaction_gains = calculate_gains(&transaction, &position_state);

            transaction.set_position_state(Some(position_state));
            transaction.set_transaction_gains(Some(transaction_gains));

            self.transactions.push(transaction);
        }

        Ok(())
    }
}
