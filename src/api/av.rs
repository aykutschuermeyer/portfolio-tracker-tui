use anyhow::{Context, Result};
use reqwest::Client;

use super::{
    av_dto::{AvGlobalQuoteDto, AvSymbolSearchDto},
    utils::{make_request, parse_response_array, parse_response_object},
};

const BASE_URL: &str = "https://www.alphavantage.co";

pub async fn get_quote(symbol: &str, client: &Client, api_key: &str) -> Result<AvGlobalQuoteDto> {
    let params = format!("function=GLOBAL_QUOTE&symbol={}&apikey={}", symbol, api_key);
    let res = make_request(client, BASE_URL, "query", &params).await?;

    if let Some(Ok(note)) = res
        .get("Information")
        .map(|v| serde_json::from_value::<String>(v.clone()))
    {
        if note.to_lowercase().contains("rate limit") {
            return Err(anyhow::anyhow!("Rate limit exceeded"));
        }
    }

    let global_quote = res
        .get("Global Quote")
        .with_context(|| "Failed to find 'Global Quote' in the response")?;

    parse_response_object::<AvGlobalQuoteDto>(
        global_quote.clone(),
        &format!("Failed to parse Alpha Vantage quote for {}", symbol),
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
        .with_context(|| "Failed to find 'bestMatches' in the response")?;

    parse_response_array::<AvSymbolSearchDto>(
        best_matches.clone(),
        &format!("Failed to parse Alpha Vantage symbol {}", &symbol),
    )
    .await
}
