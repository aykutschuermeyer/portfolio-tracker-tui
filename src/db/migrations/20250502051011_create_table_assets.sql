CREATE TABLE IF NOT EXISTS assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_name TEXT,
    ticker1_id INTEGER NOT NULL REFERENCES tickers(id),
    ticker2_id INTEGER REFERENCES tickers(id),
    ticker3_id INTEGER REFERENCES tickers(id),
    isin TEXT,
    sector TEXT,
    industry TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
)