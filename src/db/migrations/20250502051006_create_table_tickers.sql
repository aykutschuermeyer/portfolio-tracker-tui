CREATE TABLE IF NOT EXISTS tickers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    asset_id INTEGER REFERENCES assets(id),
    currency TEXT NOT NULL,
    exchange TEXT NOT NULL,
    last_price REAL,
    last_price_updated_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(symbol)
)
