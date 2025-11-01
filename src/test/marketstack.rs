#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::api::marketstack;

    const SYMBOL: &str = "BABA";

    #[tokio::test]
    async fn search_symbol_works() {
        let client = Client::new();
        let api_key = std::env::var("MARKETSTACK_API_KEY").unwrap();
        let result = marketstack::search_symbol(SYMBOL, &client, &api_key)
            .await
            .unwrap();

        assert_eq!(result.symbol(), SYMBOL);
    }

    #[tokio::test]
    async fn get_quote_works() {
        let client = Client::new();
        let api_key = std::env::var("MARKETSTACK_API_KEY").unwrap();
        let result = marketstack::get_quote(SYMBOL, &client, &api_key)
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().symbol(), SYMBOL);
    }
}
