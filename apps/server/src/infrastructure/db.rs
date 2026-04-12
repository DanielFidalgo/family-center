use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::str::FromStr;

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let opts = PgConnectOptions::from_str(database_url)?
        .options([("search_path", "family_center")])
        .statement_cache_capacity(0);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(opts)
        .await?;

    Ok(pool)
}
