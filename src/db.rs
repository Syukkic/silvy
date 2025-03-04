use crate::config::Config;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::{fs, path::Path};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self, sqlx::Error> {
        let db_path = Path::new(path);
        if !db_path.exists() {
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    sqlx::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create directory: {}", e),
                    ))
                })?
            }
            fs::File::create(db_path).map_err(|e| {
                sqlx::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create database file: {}", e),
                ))
            })?;
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", path))
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rss_feeds (
               rssurl       VARCHAR(1024) PRIMARY KEY NOT NULL, 
               url          VARCHAR(1024) NOT NULL, 
               title        VARCHAR(1024) NOT NULL, 
               is_rtl       INTEGER(1) NOT NULL DEFAULT 0, 
               etag         VARCHAR(128) NOT NULL DEFAULT ""
        );
            CREATE INDEX IF NOT EXISTS idx_rssurl ON rss_feeds(rssurl);
        "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rss_items (
                id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                guid        VARCHAR(64) NOT NULL,
                title       VARCHAR(1024) NOT NULL,
                author      VARCHAR(1024) NOT NULL,
                url         VARCHAR(1024) NOT NULL,
                feedurl     VARCHAR(1024) NOT NULL,
                pub_date    INTEGER NOT NULL,
                content     TEXT NOT NULL,
                unread      INTEGER(1) NOT NULL
        );
            CREATE INDEX idx_guid ON rss_items(guid);
            CREATE INDEX idx_feedurl ON rss_items(feedurl);
        "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn with_config(config: &Config) -> Result<Self, sqlx::Error> {
        Self::new(config.db_path.to_str().expect("Invalid database path")).await
    }
}
