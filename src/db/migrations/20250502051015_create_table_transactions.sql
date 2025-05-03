CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_no INTEGER NOT NULL,
    date DATETIME NOT NULL,
    transaction_type TEXT NOT NULL,
    ticker_id INTEGER REFERENCES tickers(id),
    broker TEXT NOT NULL,
    currency TEXT NOT NULL,
    exchange_rate TEXT NOT NULL,
    quantity TEXT NOT NULL,
    price TEXT NOT NULL,
    fees TEXT NOT NULL,
    cumulative_units TEXT NOT NULL,
    cumulative_cost TEXT NOT NULL,
    cost_of_units_sold TEXT NOT NULL,
    realized_gains TEXT NOT NULL,
    dividends_collected TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(transaction_no)
)