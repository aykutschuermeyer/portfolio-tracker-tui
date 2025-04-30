use std::env;

use portfolio_tracker_tui::app::{App, Portfolio};
use sqlx::sqlite;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_options = sqlite::SqliteConnectOptions::new()
        .filename("portfolio.db")
        .create_if_missing(true);

    let connection = sqlite::SqlitePool::connect_with(db_options).await?;
    let fmp_api_key = env::var("FMP_API_KEY").expect("Missing API key");

    let mut portfolio = Portfolio::new(String::from("EUR"), connection, fmp_api_key);

    portfolio.initialize_database_tables().await;

    portfolio
        .import_transactions("sample_data/transactions.csv")
        .await?;

    let mut app = App::new(portfolio);
    app.run()?;

    Ok(())
}
