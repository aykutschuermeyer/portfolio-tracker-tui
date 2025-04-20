use portfolio_tracker_tui::app::Portfolio;

#[tokio::main]
async fn main() {
    let mut portfolio = Portfolio::new();

    portfolio
        .import_transactions("sample_data/transactions.csv")
        .await;

    portfolio.calculate_positions();
}
