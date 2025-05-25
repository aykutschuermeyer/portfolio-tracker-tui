use anyhow::Result;
use reqwest::Client;

use super::{
    frank_dto::FrankForexDto,
    utils::{make_request, parse_response_object},
};

pub async fn get_forex_history(
    from_currency: &str,
    to_currency: &str,
    date: &str,
    client: &Client,
) -> Result<FrankForexDto> {
    let params = format!("from={}&to={}", from_currency, to_currency);
    let res = make_request(client, "https://api.frankfurter.app", &date, &params).await?;
    parse_response_object::<FrankForexDto>(
        res,
        &format!(
            "No exchange rates for date {} from {} to {}",
            date, from_currency, to_currency
        ),
    )
    .await
}
