CREATE TABLE IF NOT EXISTS tickers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker_name TEXT NOT NULL,
    currency TEXT NOT NULL,
    exchange TEXT NOT NULL,
    last_price REAL,
    last_price_updated_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
)