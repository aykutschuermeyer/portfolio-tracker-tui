use std::{env, path::Path};

use portfolio_tracker_tui::app::{App, Portfolio};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePool},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "portfolio.db";
    let db_connect_options = SqliteConnectOptions::new()
        .filename(database_url)
        .create_if_missing(true);

    let connection = SqlitePool::connect_with(db_connect_options).await?;
    let migrator = Migrator::new(Path::new("./src/db/migrations")).await?;

    migrator.run(&connection).await?;

    let api_key_av = env::var("AV_API_KEY").expect("Missing API key");
    let api_key_fmp = env::var("FMP_API_KEY").expect("Missing API key");

    let mut portfolio = Portfolio::new(String::from("EUR"), connection, api_key_av, api_key_fmp);

    //portfolio
    //    .import_transactions("sample_data/transactions.csv")
    //    .await?;

    //portfolio.update_prices().await?;

    portfolio.set_holdings().await?;

    let mut app = App::new(portfolio);
    app.run()?;

    Ok(())
}
