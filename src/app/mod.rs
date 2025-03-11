pub mod operation;
pub mod ui;

use crate::db::{
    Database,
    models::{Feed, Item},
};
use anyhow::Ok;
use crossterm::event;
use operation::handle_key_events;
use ratatui::{
    Terminal,
    prelude::Backend,
    widgets::{ListState, TableState},
};
use ui::display;

#[derive(Debug, Clone)]
pub enum Route {
    FeedsList,
    ItemsList,
    ArticleView,
}

#[derive(Debug, Clone)]
pub struct App {
    pub db: Database,
    pub route: Route,
    pub feeds: Vec<Feed>,
    pub items: Vec<Item>,
    pub feeds_state: TableState,
    pub items_state: ListState,
    pub current_article: Option<String>,
    pub article_scroll: usize,
    pub quit: bool,
}

impl App {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            route: Route::FeedsList,
            feeds: Vec::new(),
            items: Vec::new(),
            feeds_state: TableState::default(),
            items_state: ListState::default(),
            current_article: None,
            article_scroll: 0,
            quit: false,
        }
    }

    pub async fn load_feeds(&mut self) -> anyhow::Result<()> {
        self.feeds = self.db.get_all_feeds().await?;
        self.feeds_state.select(Some(0));
        Ok(())
    }

    pub async fn load_items(&mut self) -> anyhow::Result<()> {
        if let Some(idx) = self.feeds_state.selected() {
            let feed = &self.feeds[idx];
            self.items = self.db.get_items(&feed.url).await?;

            self.items_state.select(Some(0));
        }

        Ok(())
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), anyhow::Error> {
    loop {
        terminal.draw(|f| display(app, f))?;
        if event::poll(std::time::Duration::from_millis(200))? {
            if let event::Event::Key(key) = event::read()? {
                handle_key_events(app, key)
            }
        }

        if app.quit {
            break;
        }
    }

    Ok(())
}
