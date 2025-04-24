use anyhow::{Error, Result};
use reqwest::Client;
use serde_json::Value;

pub async fn make_request(
    client: &Client,
    base_url: &str,
    endpoint: &str,
    api_key: &str,
) -> Result<Value> {
    let url = format!("{}/{}&apikey={}", base_url, endpoint, api_key);
    let res = client.get(&url).send().await?;

    if !res.status().is_success() {
        return Err(Error::msg(format!("Request failed: {}", res.status())));
    }

    let text = res.text().await?;
    let data = serde_json::from_str::<Vec<Value>>(&text)?;

    Ok(Value::Array(data))
}
