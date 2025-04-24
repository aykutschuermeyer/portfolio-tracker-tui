use anyhow::{Error, Result};
use reqwest::Client;
use serde_json::Value;

use super::{
    base,
    dto::{FmpQuoteDto, FmpSearchSymbolDto},
};

const BASE_URL: &str = "https://financialmodelingprep.com/stable";

pub async fn search_symbol(
    symbol: &str,
    exchange: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<FmpSearchSymbolDto>> {
    let endpoint = format!("search-symbol?query={}&exchange={}", symbol, exchange);
    let res = base::make_request(client, BASE_URL, &endpoint, api_key).await?;

    match res {
        Value::Array(data) => {
            let symbols: Vec<FmpSearchSymbolDto> = data
                .into_iter()
                .filter_map(|item| serde_json::from_value(item).ok())
                .collect();

            match symbols.is_empty() {
                true => Err(Error::msg(format!(
                    "No symbols found for query {symbol} on exchange {exchange}"
                ))),
                false => Ok(symbols),
            }
        }
        _ => Err(Error::msg("Unexpected API response format")),
    }
}

pub async fn get_quote(
    ticker_symbol: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<FmpQuoteDto>> {
    let endpoint = format!("quote?symbol={}", ticker_symbol);
    let res = base::make_request(client, BASE_URL, &endpoint, api_key).await?;

    match res {
        Value::Array(data) => {
            let quotes: Vec<FmpQuoteDto> = data
                .into_iter()
                .filter_map(|item| serde_json::from_value(item).ok())
                .collect();

            match quotes.is_empty() {
                true => Err(Error::msg(format!(
                    "No quote data found for ticker {ticker_symbol}"
                ))),
                false => Ok(quotes),
            }
        }
        _ => Err(Error::msg("Unexpected API response format")),
    }
}
