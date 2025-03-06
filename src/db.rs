use crate::{
    config::Config,
    models::{Feed, Item},
};
use anyhow::{Context, Ok, Result};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", path))
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rss_feeds (
               rssurl       VARCHAR(1024) PRIMARY KEY NOT NULL, 
               url          VARCHAR(1024) UNIQUE NOT NULL, 
               title        VARCHAR(1024) NOT NULL
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
                unread      INTEGER(1) NOT NULL,
                FOREIGN KEY (feedurl) REFERENCES rss_feeds(rssurl)
        );
            CREATE INDEX IF NOT EXISTS idx_guid ON rss_items(guid);
            CREATE INDEX IF NOT EXISTS idx_feedurl ON rss_items(feedurl);
        "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn with_config(config: &Config) -> Result<Self> {
        let db_path = config.db_path.to_str().context("Invalid database path")?;
        Self::new(db_path).await
    }

    pub async fn insert_feed(&self, feed: &Feed) -> Result<()> {
        sqlx::query(r#"INSERT OR IGNORE INTO rss_feeds (rssurl, url, title) VALUES (?1, ?2, ?3)"#)
            .bind(feed.rssurl.to_string())
            .bind(feed.url.to_string())
            .bind(feed.title.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_item(&self, item: &Item) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO rss_items (guid, title, author, url, feedurl, pub_date, content, unread) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&item.guid)
        .bind(&item.title)
        .bind(&item.author)
        .bind(&item.url)
        .bind(&item.feedurl)
        .bind(&item.pub_date)
        .bind(&item.content)
        .bind(item.unread)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
