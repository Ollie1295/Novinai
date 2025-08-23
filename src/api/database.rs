pub struct DatabaseConfig;
impl Default for DatabaseConfig { fn default() -> Self { Self } }
pub async fn initialize_database(_: DatabaseConfig) -> Result<sqlx::SqlitePool, anyhow::Error> {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
    Ok(pool)
}
