use anyhow::Result;
use config::Config;
use db::Database;
use feeds::UrlEntry;

mod config;
mod db;
mod feeds;
mod fetcher;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new()?;
    let feeds = UrlEntry::from_file(&config)?;
    Database::with_config(&config).await?;

    Ok(())
}
