use db::Database;

mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = Database::new("cache.db").await?;

    Ok(())
}

