use std::{error::Error, fs, path::Path};

use portfolio_tracker_tui::app::{App, Portfolio};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePool},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db_dir = shellexpand::tilde("~/.local/share/portfolio-tracker-tui");
    fs::create_dir_all(db_dir.as_ref())?;
    let database_url = format!("{}/portfolio.db", db_dir);
    let db_connect_options = SqliteConnectOptions::new()
        .filename(database_url)
        .create_if_missing(true);
    let connection = SqlitePool::connect_with(db_connect_options).await?;
    let migrator = Migrator::new(Path::new("./src/db/migrations")).await?;

    migrator.run(&connection).await?;

    let mut portfolio = Portfolio::new(String::from("EUR"), connection);

    portfolio.set_holdings().await?;

    let mut app = App::new(portfolio);
    let csv_path = "~/.config/portfolio-tracker-tui/transactions.csv";
    app.run(&csv_path).await?;

    Ok(())
}
