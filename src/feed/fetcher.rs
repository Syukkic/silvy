use anyhow::{Context, Result, bail};
use chrono::{DateTime, TimeZone, Utc};
use feed_rs::model::Feed;
use feed_rs::parser as rss_parser;
use url::Url;

use crate::db::models::{Feed as db_Feed, Item};
use ammonia::Builder;
use std::time::Duration;

pub struct FeedFetcher {
    client: reqwest::Client,
    retries: u32,
}

impl FeedFetcher {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(30))
                .build()
                .context("Failed to establish HTTP connection")?,
            retries: 3,
        })
    }

    pub async fn try_fetch(&self, url: &str) -> Result<(db_Feed, Vec<Item>)> {
        for attempt in 1..=self.retries {
            match self.process_feed_from_url(url).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == self.retries {
                        return Err(e);
                    }
                    let wait_time = Duration::from_secs(2_u64.pow(attempt));
                    println!(
                        "Attempt {}/{} failed, retrying in {:?}...",
                        attempt + 1,
                        self.retries,
                        wait_time
                    );
                }
            }
        }
        bail!(
            "Failed to fetch {} RSS feed after {} retries.",
            url,
            self.retries
        )
    }

    async fn process_feed_from_url(&self, url: &str) -> Result<(db_Feed, Vec<Item>)> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/rss+xml, application/atom+xml")
            .send()
            .await
            .context("Failed to establish HTTP request")?;

        if !response.status().is_success() {
            bail!("HTTP error: {} (URL: {})", response.status(), &url)
        }

        let content_type = response
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("xml") {
            bail!("Non-XML format content: {} (URL: {})", content_type, url)
        }

        let bytes = response.bytes().await.context("Failed to read content")?;
        let parsed_feed = rss_parser::parse(&bytes[..])
            .context(format!("Failed to parse feed data from {}", url))?;

        let feed = self.normalize_feed(&parsed_feed, url)?;
        let items = self.normalize_items(parsed_feed, url)?;

        Ok((feed, items))
    }

    fn normalize_feed(&self, feed: &Feed, url: &str) -> Result<db_Feed> {
        Ok(db_Feed {
            rssurl: url.to_string(),
            url: url.to_string(),
            title: feed
                .title
                .as_ref()
                .map(|t| t.content.clone())
                .unwrap_or_default(),
            unread_count: 0,
        })
    }

    fn normalize_items(&self, feed: Feed, url: &str) -> Result<Vec<Item>> {
        let mut items: Vec<Item> = Vec::new();

        for entry in feed.entries {
            let raw_content = entry.content.and_then(|c| c.body).unwrap_or_default();
            let cleaned_content = self.sanitize_content(&raw_content, url)?;
            let pub_date = entry
                .published
                .or(entry.updated)
                .map(|dt| dt.timestamp())
                .unwrap_or(0);

            items.push(Item {
                id: 0,
                guid: entry.id,
                title: entry.title.map(|t| t.content).unwrap_or_default(),
                author: entry
                    .authors
                    .first()
                    .map(|a| a.name.clone())
                    .unwrap_or_default(), // Maybe more than one ?
                url: entry
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .unwrap_or_default(),
                feedurl: url.to_string(),
                pub_date,
                content: cleaned_content,
                unread: 1,
            });
        }

        Ok(items)
    }

    pub fn sanitize_content(&self, html: &str, base_url: &str) -> Result<String> {
        // https://docs.rs/ammonia/latest/ammonia/struct.Builder.html#method.link_rel
        let parsed_url = Url::parse(base_url);

        if let Ok(url) = parsed_url {
            let mut cleaner = Builder::default();
            cleaner.url_relative(ammonia::UrlRelative::RewriteWithBase(url));
            let sanitized_html = cleaner.clean(html).to_string();
            let text = html2text::from_read(sanitized_html.as_bytes(), sanitized_html.len())
                .context("Failed to convert HTML to TEXT")?;

            Ok(text.trim().to_string())
        } else {
            Err(anyhow::anyhow!("Invalid URL").into())
        }
        // cleaner.clean(html).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Ok, Result};
    use mockito::{Server, ServerGuard};

    async fn setup_mock_server() -> ServerGuard {
        Server::new_async().await
    }

    async fn test_fetch(
        fetcher: &FeedFetcher,
        server: &ServerGuard,
        path: &str,
    ) -> Result<(db_Feed, Vec<Item>)> {
        fetcher
            .process_feed_from_url(&format!("{}/{}", server.url(), path))
            .await
    }

    #[tokio::test]
    async fn test_fetch_feed_success() {
        let mut server = setup_mock_server().await;
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

        let fetcher = FeedFetcher::new().unwrap();
        let result = test_fetch(&fetcher, &server, "rss").await;

        assert!(result.is_ok());
        let (feed, items) = result.unwrap();

        assert_eq!(feed.title, "Example Feed");
        assert_eq!(feed.unread_count, 0);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Example Item");
        assert_eq!(items[0].url, "http://example.com/item");
    }

    #[tokio::test]
    async fn test_fetch_feed_http_error() {
        let mut server = setup_mock_server().await;
        let _m = server.mock("GET", "/rss").with_status(404).create();

        let fetcher = FeedFetcher::new().unwrap();
        let result = test_fetch(&fetcher, &server, "rss").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP error"));
    }

    #[tokio::test]
    async fn test_fetch_feed_non_xml() {
        let mut server = setup_mock_server().await;
        let _m = server
            .mock("GET", "/rss")
            .with_status(200)
            .with_header("Content-Type", "text/html")
            .with_body("<html>Not XML</html>")
            .create();

        let fetcher = FeedFetcher::new().unwrap();
        let result = test_fetch(&fetcher, &server, "rss").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Non-XML format content"));
    }

    #[test]
    fn test_sanitize_content() -> Result<()> {
        let fetcher = FeedFetcher::new().unwrap();
        let html = r#"<div><p>Hello, <strong>world</strong>!</p><script>alert("XSS");</script><a href="/about" onclick="alert('clicked')">About</a></div>"#;

        let cleaned = fetcher.sanitize_content(html, "https://example.com")?;
        assert_eq!(
            cleaned,
            r"Hello, **world**!

[About][1]

[1]: https://example.com/about"
        );

        Ok(())
    }
}
