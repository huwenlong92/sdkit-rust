use crate::sqlx::postgres::PgPoolOptions;

pub type PostgresDb = crate::sqlx::PgPool;

#[derive(Clone)]
pub struct PostgresConnector;

impl PostgresConnector {
    pub async fn connect(url: &str) -> Result<PostgresDb, String> {
        PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
            .map_err(|e| e.to_string())
    }
}
