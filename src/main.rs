use std::{env, error::Error, path::Path};

use clap::Parser;
use portfolio_tracker_tui::app::{App, Portfolio};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePool},
};

#[derive(Parser)]
#[command(name = "portfolio-tracker-tui")]
#[command(about = "A terminal-based portfolio tracker")]
struct Args {
    #[arg(
        long,
        default_value = "~/.config/portfolio-tracker-tui/transactions.csv"
    )]
    csv_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let database_url = shellexpand::tilde("~/.local/share/portfolio-tracker-tui/portfolio.db");
    let db_connect_options = SqliteConnectOptions::new()
        .filename(database_url.as_ref())
        .create_if_missing(true);
    let connection = SqlitePool::connect_with(db_connect_options).await?;
    let migrator = Migrator::new(Path::new("./src/db/migrations")).await?;

    migrator.run(&connection).await?;

    let api_key_av = env::var("AV_API_KEY").expect("Missing API key");
    let api_key_fmp = env::var("FMP_API_KEY").expect("Missing API key");

    let mut portfolio = Portfolio::new(String::from("EUR"), connection, api_key_av, api_key_fmp);

    portfolio.set_holdings().await?;

    let mut app = App::new(portfolio);
    app.run(&args.csv_path).await?;

    Ok(())
}
