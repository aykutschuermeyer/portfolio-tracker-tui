use anyhow::Result;
use reqwest::Client;

use super::{
    av_dto::{AvGlobalQuoteDto, AvSymbolSearchDto},
    utils::{make_request, parse_response_array, parse_response_object},
};

const BASE_URL: &str = "https://www.alphavantage.co";

pub async fn get_quote(symbol: &str, client: &Client, api_key: &str) -> Result<AvGlobalQuoteDto> {
    let params = format!("function=GLOBAL_QUOTE&symbol={}&apikey={}", symbol, api_key);
    let res = make_request(client, BASE_URL, "query", &params).await?;

    let global_quote = res
        .get("Global Quote")
        .ok_or_else(|| anyhow::anyhow!("Failed to find 'Global Quote' in the response"))?;

    parse_response_object::<AvGlobalQuoteDto>(
        global_quote.clone(),
        &format!("No results for symbol {}", symbol),
    )
    .await
}

pub async fn search_symbol(
    symbol: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<AvSymbolSearchDto>> {
    let params = format!(
        "function=SYMBOL_SEARCH&keywords={}&apikey={}",
        symbol, api_key
    );
    let res = make_request(client, BASE_URL, "query", &params).await?;

    let best_matches = res
        .get("bestMatches")
        .ok_or_else(|| anyhow::anyhow!("Failed to find 'bestMatches' in the response"))?;

    parse_response_array::<AvSymbolSearchDto>(
        best_matches.clone(),
        &format!("No results for symbol {}", &symbol),
    )
    .await
}
