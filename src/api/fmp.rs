use anyhow::Result;
use reqwest::Client;

use super::{
    dto::{FmpQuoteDto, FmpSearchSymbolDto},
    utils::{make_request, parse_response_array},
};

const BASE_URL: &str = "https://financialmodelingprep.com/stable";

pub async fn search_symbol(
    symbol: &str,
    exchange: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<FmpSearchSymbolDto>> {
    let endpoint = format!("search-symbol?query={}&exchange={}", symbol, exchange);
    let res = make_request(client, BASE_URL, &endpoint, api_key).await?;
    parse_response_array::<FmpSearchSymbolDto>(
        res,
        &format!("No symbols found for query {symbol} on exchange {exchange}"),
    )
    .await
}

pub async fn get_quote(
    ticker_symbol: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<FmpQuoteDto>> {
    let endpoint = format!("quote?symbol={}", ticker_symbol);
    let res = make_request(client, BASE_URL, &endpoint, api_key).await?;
    parse_response_array::<FmpQuoteDto>(
        res,
        &format!("No quote data found for ticker {ticker_symbol}"),
    )
    .await
}
