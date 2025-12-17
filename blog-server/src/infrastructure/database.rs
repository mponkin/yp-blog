use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::trace;

use crate::domain::error::AppError;

pub async fn init_db_connection(url: &str) -> Result<PgPool, AppError> {
    trace!("Creating connection pool for DB at {url}");
    const MAX_CONNECTIONS: u32 = 5;
    let pool = PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect(url)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    trace!("Running migrations");
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
