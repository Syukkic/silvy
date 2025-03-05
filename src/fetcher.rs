use crate::db::Database;
use crate::models::{Feed, Item};
use anyhow::Result;
use rss::Channel;
use url::Url;

pub async fn fetch_rss(feed_url: &Url, db: &Database) -> Result<()> {
    let content = reqwest::get(feed_url.clone()).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    let feed = Feed::new(
        feed_url.to_string(),
        channel.link.clone(),
        channel.title.clone(),
    );

    db.insert_feed(&feed).await?;

    for item in channel.items() {
        let db_item = Item {
            id: 0,
            guid: item
                .guid()
                .map(|g| g.value().to_string())
                .unwrap_or_default(),
            title: item.title().unwrap_or_default().to_string(),
            author: item.author().unwrap_or_default().to_string(),
            url: item.link().unwrap_or_default().to_string(),
            feedurl: feed_url.to_string(),
            pub_date: item.pub_date().unwrap_or_default().to_string(),
            content: item.description().unwrap_or_default().to_string(),
            unread: 1, // unread by default
        };

        db.insert_item(&db_item).await.unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use mockito::Server;
    use sqlx::sqlite::SqlitePoolOptions;
    use url::Url;

    async fn setup_db() -> Result<Database> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .unwrap();
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
            CREATE INDEX idx_guid ON rss_items(guid);
            CREATE INDEX idx_feedurl ON rss_items(feedurl);
        "#,
        )
        .execute(&pool)
        .await?;
        Ok(Database { pool })
    }

    #[tokio::test]
    async fn test_fetch_feed() {
        let mut server = Server::new_async().await;
        let mock_rss = r#"
            <rss version="2.0">
                <channel>
                    <title>Example Feed</title>
                    <link>http://example.com</link>
                    <item>
                        <title>Example Item</title>
                        <link>http://example.com/item</link>
                        <description>Example content</description>
                        <pubDate>2023-10-01T00:00:00Z</pubDate>
                        <guid>guid123</guid>
                        <author>Author</author>
                    </item>
                </channel>
            </rss>
        "#;

        let _m = server
            .mock("GET", "/rss")
            .with_status(200)
            .with_header("content-type", "application/rss+xml")
            .with_body(mock_rss)
            .create();

        let feed_url = Url::parse(&server.url()).unwrap().join("/rss").unwrap();
        let db = setup_db().await.unwrap();
        fetch_rss(&feed_url, &db).await.unwrap();

        dbg!(&server.url());
        let feed = sqlx::query_as::<_, Feed>("SELECT * FROM rss_feeds")
            .fetch_one(&db.pool)
            .await
            .unwrap();

        assert_eq!(feed.rssurl, format!("{}/rss", server.url()));
        assert_eq!(feed.title, "Example Feed");
        assert_eq!(feed.url, "http://example.com");

        let item = sqlx::query_as::<_, Item>("SELECT * FROM rss_items WHERE guid = ?")
            .bind("guid123")
            .fetch_one(&db.pool)
            .await
            .unwrap();

        assert_eq!(item.title, "Example Item");
        assert_eq!(item.url, "http://example.com/item");
        assert_eq!(item.content, "Example content");
    }
}
