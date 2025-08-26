use std::{error::Error, fs, path::Path};

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
    app.run(&args.csv_path).await?;

    Ok(())
}
