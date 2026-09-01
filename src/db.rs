use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn init_pool() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://tickets.db?mode=rwc")
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
