use portfolio_tracker_tui::app::Portfolio;

#[tokio::main]
async fn main() {
    let mut portfolio = Portfolio::new();

    let result = portfolio
        .import_transactions("sample_data/transactions.csv")
        .await
        .unwrap_or(false);

    println!("Success: {}", result);
}
