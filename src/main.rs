use portfolio_tracker_tui::app::{App, Portfolio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut portfolio = Portfolio::new();

    portfolio
        .import_transactions("sample_data/transactions.csv")
        .await?;

    let mut app = App::new(portfolio);
    app.run()?;

    Ok(())
}
