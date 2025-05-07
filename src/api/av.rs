use anyhow::Result;
use reqwest::Client;

use super::{
    av_dto::{AvGlobalQuoteDto, AvSymbolSearchDto},
    utils::{make_request, parse_response_array, parse_response_object},
};

const BASE_URL: &str = "https://www.alphavantage.co/query";

pub async fn get_quote(symbol: &str, api_key: &str, client: &Client) -> Result<AvGlobalQuoteDto> {
    let endpoint = format!(
        "?function=GLOBAL_QUOTE&symbol={}&apikey={}",
        symbol, api_key
    );
    let res = make_request(client, BASE_URL, &endpoint, api_key).await?;

    let global_quote = res
        .get("Global Quote")
        .ok_or_else(|| anyhow::anyhow!("Failed to find 'Global Quote' in the response"))?;

    parse_response_object::<AvGlobalQuoteDto>(
        global_quote.clone(),
        &format!("Failed to get data for ticker {}", symbol),
    )
    .await
}

pub async fn search_symbol(
    symbol: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<AvSymbolSearchDto>> {
    let endpoint = format!(
        "?function=SYMBOL_SEARCH&keywords={}&apikey={}",
        &symbol, api_key
    );
    let res = make_request(client, BASE_URL, &endpoint, api_key).await?;

    let best_matches = res
        .get("bestMatches")
        .ok_or_else(|| anyhow::anyhow!("Failed to find 'bestMatches' in the response"))?;

    parse_response_array::<AvSymbolSearchDto>(
        best_matches.clone(),
        &format!("Failed to get data for ticker {}", &symbol),
    )
    .await
}
