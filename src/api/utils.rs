use anyhow::{Error, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub async fn make_request(
    client: &Client,
    base_url: &str,
    endpoint: &str,
    api_key: &str,
) -> Result<Value> {
    let url = format!("{}/{}&apikey={}", base_url, endpoint, api_key);
    let res = client.get(&url).send().await?;

    // println!("{:#?}", url);

    if !res.status().is_success() {
        return Err(Error::msg(format!("Request failed: {}", res.status())));
    }

    let text = res.text().await?;
    let data = serde_json::from_str::<Value>(&text)?;

    Ok(data)
}

pub async fn parse_response_array<T>(data: Value, error_msg: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    match data {
        Value::Array(items) => {
            let result: Vec<T> = items
                .into_iter()
                .filter_map(|item| serde_json::from_value(item).ok())
                .collect();

            if result.is_empty() {
                Err(Error::msg(error_msg.to_string()))
            } else {
                Ok(result)
            }
        }
        _ => Err(Error::msg("Unexpected API response format: not an array")),
    }
}

pub async fn parse_response_object<T>(data: Value, error_msg: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    match data {
        Value::Object(obj) => serde_json::from_value(Value::Object(obj))
            .map_err(|_| Error::msg(error_msg.to_string())),
        _ => Err(Error::msg("Unexpected API response format: not an object")),
    }
}
