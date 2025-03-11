use anyhow::Context;
use app::{App, run_app};
use config::Config;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use db::Database;
use feed::{UrlEntry, fetcher::FeedFetcher};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

mod app;
mod config;
mod db;
mod feed;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;

    let db = Database::with_config(&config).await?;
    let mut app = App::new(db.clone());

    let feeds = UrlEntry::from_file(&config)?;
    let fetcher = FeedFetcher::new()?;

    // for feed in feeds {
    //     match fetcher.try_fetch(feed.url.as_str()).await {
    //         Ok((feed, items)) => {
    //             println!("成功抓取 {} 条内容: {}", items.len(), feed.url);
    //             db.update_feed_and_item(&feed, &items).await?;
    //         }
    //         Err(e) => {
    //             eprintln!("抓取失败: {}，错误: {}", feed.url, e);
    //         }
    //     }
    // }

    enable_raw_mode().context("Failed to start terminal")?;
    let mut stderr = io::stdout();
    execute!(stderr, EnterAlternateScreen).context("Failed to enter alternative screen")?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    app.load_feeds().await?;
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(())
}
