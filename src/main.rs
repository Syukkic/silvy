use config::Config;
use db::Database;

mod config;
mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;
    let _ = Database::with_config(&config).await?;

    Ok(())
}
