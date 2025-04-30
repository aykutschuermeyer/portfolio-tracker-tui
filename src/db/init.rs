use sqlx::sqlite::SqliteQueryResult;

pub async fn create_tickers(
    connection: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<SqliteQueryResult, sqlx::Error> {
    sqlx::query(
        r#"
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
        "#,
    )
    .execute(connection)
    .await
}

pub async fn create_assets(
    connection: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<SqliteQueryResult, sqlx::Error> {
    sqlx::query(
        r#"
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
        "#,
    )
    .execute(connection)
    .await
}

pub async fn create_transactions(
    connection: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<SqliteQueryResult, sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            transaction_date TEXT NOT NULL,
            asset_id INTEGER REFERENCES assets(id),
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
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(connection)
    .await
}
