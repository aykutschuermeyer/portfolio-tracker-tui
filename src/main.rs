use anyhow::Result;
use portfolio_tracker_tui::services::portfolio_tracker_service::PortfolioTrackerService;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("ALPHA_VANTAGE_API_KEY")
        .expect("Missing environment variable ALPHA_VANTAGE_API_KEY");

    let service = PortfolioTrackerService::new(api_key);
    let transactions = service.read_transactions("sample_data/transactions.csv")?;

    for transaction in transactions {
        println!("Transaction: {:?}", transaction);
    }

    Ok(())
}
