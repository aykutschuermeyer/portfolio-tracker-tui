use anyhow::Result;
use reqwest::Client;

use super::{
    fmp_dto::{FmpQuoteDto, FmpQuoteHistoryDto, FmpSearchSymbolDto},
    utils::{make_request, parse_response_array},
};

const BASE_URL: &str = "https://financialmodelingprep.com/stable";

pub async fn search_symbol(
    symbol: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<FmpSearchSymbolDto>> {
    let params = format!("query={}&apikey={}", symbol, api_key);
    let res = make_request(client, BASE_URL, "search-symbol", &params).await?;
    parse_response_array::<FmpSearchSymbolDto>(res, &format!("No results for symbol {symbol}"))
        .await
}

pub async fn get_quote(symbol: &str, client: &Client, api_key: &str) -> Result<Vec<FmpQuoteDto>> {
    let params = format!("symbol={}&apikey={}", symbol, api_key);
    let res = make_request(client, BASE_URL, "quote", &params).await?;
    parse_response_array::<FmpQuoteDto>(res, &format!("No Results for symbol {symbol}")).await
}

pub async fn get_quote_history(
    symbol: &str,
    start_date: &str,
    end_date: &str,
    client: &Client,
    api_key: &str,
) -> Result<Vec<FmpQuoteHistoryDto>> {
    let params = format!(
        "symbol={}&from={}&to={}&apikey={}",
        symbol, start_date, end_date, api_key
    );
    let res = make_request(client, BASE_URL, "historical-price-eod/light", &params).await?;
    parse_response_array::<FmpQuoteHistoryDto>(
        res,
        &format!("No results for symbol {symbol} from {start_date} to {end_date}"),
    )
    .await
}
