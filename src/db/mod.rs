use anyhow::{Context, Ok, Result};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::config::Config;
use models::{Feed, Item};

pub mod models;

#[derive(Debug, Clone)]
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
               title        VARCHAR(1024) NOT NULL,
               unread_count INT
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
                FOREIGN KEY (feedurl) REFERENCES rss_feeds(rssurl) ON DELETE CASCADE
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

    pub async fn update_feed_and_item(&self, feed: &Feed, items: &[Item]) -> Result<()> {
        let tx = self.pool.begin().await?;

        sqlx::query(r#"INSERT OR IGNORE INTO rss_feeds (rssurl, url, title) VALUES (?1, ?2, ?3)"#)
            .bind(&feed.rssurl)
            .bind(&feed.url)
            .bind(&feed.title)
            .bind(&feed.unread_count)
            .execute(&self.pool)
            .await?;

        for item in items {
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
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        let feeds: Vec<Feed> = sqlx::query_as(
            r#"SELECT f.*, 
                (SELECT COUNT(*) FROM rss_items 
                 WHERE feedurl = f.rssurl AND unread = 1) as unread_count
             FROM rss_feeds f
             ORDER BY title COLLATE NOCASE"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(feeds)
    }

    pub async fn get_items(&self, feed_url: &str) -> Result<Vec<Item>> {
        let items: Vec<Item> =
            sqlx::query_as(r#"SELECT * FROM rss_items WHERE feedurl = ? ORDER BY pub_date DESC"#)
                .bind(feed_url)
                .fetch_all(&self.pool)
                .await?;
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_feed_and_items() {
        let db = Database::new(":memory:").await.unwrap();
        let feed = Feed {
            rssurl: "http://example.com/feed".to_string(),
            url: "http://example.com".to_string(),
            title: "Example Feed".to_string(),
            unread_count: 0,
        };

        let items = vec![Item {
            id: 0,
            guid: "guid123".to_string(),
            title: "Example Item".to_string(),
            author: "Author".to_string(),
            url: "http://example.com/item".to_string(),
            feedurl: "http://example.com/feed".to_string(),
            pub_date: 1735827065,
            content: "Example content".to_string(),
            unread: 1,
        }];

        db.update_feed_and_item(&feed, &items).await.unwrap();

        let stored_feed: Feed = sqlx::query_as("SELECT * FROM rss_feeds WHERE rssurl = ?")
            .bind(&feed.rssurl)
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(stored_feed.title, "Example Feed");

        let stored_items: Vec<Item> = sqlx::query_as("SELECT * FROM rss_items WHERE feedurl = ?")
            .bind(&feed.rssurl)
            .fetch_all(&db.pool)
            .await
            .unwrap();

        assert_eq!(stored_items.len(), 1);
        assert_eq!(stored_items[0].title, "Example Item");
    }
}
