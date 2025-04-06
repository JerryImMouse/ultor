use crate::error::Error;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::PathBuf;
#[derive(Debug)]
pub struct BotDatabaseService {
    #[allow(dead_code)]
    inner: SqlitePool,
}

impl BotDatabaseService {
    pub async fn new(database_path: String, migrations_path: String) -> Result<Self, Error> {
        let database_path = PathBuf::from(database_path);
        let migrations_path = PathBuf::from(migrations_path);

        let options = SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename(database_path);

        let pool = SqlitePoolOptions::new().connect_lazy_with(options);

        let migrator = Migrator::new(migrations_path).await?;
        migrator.run(&pool).await?;

        Ok(Self { inner: pool })
    }

    // implement your own methods here
    // if you want to modify database structure -> look at migrations directory at the root of the project
}
