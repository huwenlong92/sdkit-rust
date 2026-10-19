use std::path::Path;

#[cfg(feature = "postgres")]
use super::postgres::{PostgresConnector, PostgresDb};
#[cfg(feature = "sqlite")]
use super::sqlite::{self, SqliteDb};
use super::{DatabaseConfig, DatabaseDriver};

#[derive(Clone)]
pub enum Database {
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteDb),
    #[cfg(feature = "postgres")]
    Postgres(PostgresDb),
}

impl Database {
    pub async fn connect(config: &DatabaseConfig, sqlite_path: &Path) -> Result<Self, String> {
        match config.driver {
            DatabaseDriver::Sqlite => connect_sqlite(sqlite_path).await,
            DatabaseDriver::Postgres => connect_postgres(config).await,
        }
    }

    pub fn driver(&self) -> DatabaseDriver {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(_) => DatabaseDriver::Sqlite,
            #[cfg(feature = "postgres")]
            Self::Postgres(_) => DatabaseDriver::Postgres,
        }
    }

    #[cfg(feature = "sqlite")]
    pub fn sqlite(&self) -> Result<SqliteDb, String> {
        match self {
            Self::Sqlite(db) => Ok(db.clone()),
            #[cfg(feature = "postgres")]
            Self::Postgres(_) => Err("sqlite repository cannot use postgres database".to_string()),
        }
    }

    #[cfg(feature = "postgres")]
    pub fn postgres(&self) -> Result<PostgresDb, String> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(_) => Err("postgres repository cannot use sqlite database".to_string()),
            Self::Postgres(db) => Ok(db.clone()),
        }
    }
}

#[cfg(feature = "sqlite")]
async fn connect_sqlite(path: &Path) -> Result<Database, String> {
    sqlite::connect(path).await.map(Database::Sqlite)
}

#[cfg(not(feature = "sqlite"))]
async fn connect_sqlite(_path: &Path) -> Result<Database, String> {
    Err("sqlite database driver is not enabled".to_string())
}

#[cfg(feature = "postgres")]
async fn connect_postgres(config: &DatabaseConfig) -> Result<Database, String> {
    let url = config
        .url
        .as_deref()
        .ok_or_else(|| "postgres database url is required".to_string())?;
    PostgresConnector::connect(url)
        .await
        .map(Database::Postgres)
}

#[cfg(not(feature = "postgres"))]
async fn connect_postgres(_config: &DatabaseConfig) -> Result<Database, String> {
    Err("postgres database driver is not enabled; rebuild with --features postgres".to_string())
}
