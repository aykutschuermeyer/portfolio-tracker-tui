CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_no INTEGER NOT NULL,
    transaction_date DATETIME NOT NULL,
    transaction_type TEXT NOT NULL,
    ticker_id INTEGER REFERENCES tickers(id),
    broker TEXT NOT NULL,
    currency TEXT NOT NULL,
    exchange_rate REAL NOT NULL,
    quantity REAL NOT NULL,
    price REAL NOT NULL,
    fees REAL NOT NULL,
    cumulative_units REAL NOT NULL,
    cumulative_cost REAL NOT NULL,
    cost_of_units_sold REAL NOT NULL,
    realized_gains REAL NOT NULL,
    dividends_collected REAL NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(transaction_no)
)
