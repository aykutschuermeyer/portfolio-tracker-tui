use anyhow::{Context, Result};
use reqwest::Client;

use crate::api::{
    marketstack_dto::{MarketstackQuoteDto, MarketstackSearchSymbolDto},
    utils::{make_request, parse_response_array, parse_response_object},
};

const BASE_URL: &str = "https://api.marketstack.com/v2";

pub async fn get_quote(
    symbol: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<MarketstackQuoteDto>> {
    let params = format!("access_key={}&symbols={}", api_key, symbol);
    let res = make_request(client, BASE_URL, "eod/latest", &params).await?;

    let quote = res
        .get("data")
        .with_context(|| "Failed to get 'data' in response")?;

    parse_response_array::<MarketstackQuoteDto>(
        quote.clone(),
        &format!("Failed to parse Marketstack quote for {}", symbol),
    )
    .await
}

pub async fn search_symbol(
    symbol: &str,
    client: &Client,
    api_key: &str,
) -> Result<MarketstackSearchSymbolDto> {
    let params = format!("access_key={}", api_key);
    let res = make_request(client, BASE_URL, &format!("tickers/{}", symbol), &params).await?;
    parse_response_object::<MarketstackSearchSymbolDto>(
        res,
        &format!("Failed to parse Marketstack symbol {}", symbol),
    )
    .await
}
