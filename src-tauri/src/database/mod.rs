pub mod migrations;
pub mod models;
pub mod repositories;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Not found")]
    NotFound,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_path: &str) -> Result<Self, DatabaseError> {
        let database_file = Path::new(database_path);
        if let Some(parent) = database_file.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let options = SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;

        migrations::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
