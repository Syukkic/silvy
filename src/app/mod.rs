use crate::db::{
    Database,
    models::{Feed, Item},
};
use anyhow::Ok;
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Backend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap},
};

#[derive(Debug)]
pub enum Route {
    FeedsList,
    ItemsList,
    ArticleView,
}

#[derive(Debug)]
pub struct App {
    pub db: Database,
    pub route: Route,
    pub feeds: Vec<Feed>,
    pub items: Vec<Item>,
    pub feeds_state: TableState,
    pub items_state: ListState,
    pub current_article: Option<String>,
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

pub fn display(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title bar
            Constraint::Min(0),    // content section
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    let title = Paragraph::new("Silvy - Your feeds (? unread, ? total)")
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(title, chunks[0]);

    match app.route {
        Route::FeedsList => render_feeds_list(app, frame, chunks[1]),
        Route::ItemsList => render_items_list(app, frame, chunks[1]),
        Route::ArticleView => render_article(app, frame, chunks[1]),
    }
}

fn render_feeds_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let rows = app.feeds.iter().map(|feed| {
        let _ = feed.unread_count.to_string();
        Row::new(vec![feed.title.as_str(), feed.url.as_str()])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(50),
            Constraint::Length(10),
        ],
    )
    .style(Style::default().fg(Color::Magenta))
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(table, area, &mut app.feeds_state);
}

fn render_items_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|item| {
            let style = if item.unread == 1 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} - {}", item.pub_date, item.title)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Articles"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut app.items_state);
}

fn render_article(app: &mut App, frame: &mut Frame, area: Rect) {
    let content = app.current_article.as_deref().unwrap_or("No content");
    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Artitle"))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn handle_key_events(app: &mut App, key: KeyEvent) {
    match app.route {
        Route::FeedsList => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                move_selection_down_table(&mut app.feeds_state, app.feeds.len())
            }
            KeyCode::Char('k') | KeyCode::Up => move_selection_up_table(&mut app.feeds_state),
            KeyCode::Char('l') | KeyCode::Enter => {
                app.route = Route::ItemsList;
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let _ = app.load_items().await;
                    });
                });
            }
            KeyCode::Char('q') => app.quit = true,
            _ => {}
        },
        Route::ItemsList => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                move_selection_down_list(&mut app.items_state, app.items.len())
            }
            KeyCode::Char('k') | KeyCode::Up => move_selection_up_list(&mut app.items_state),
            KeyCode::Char('l') | KeyCode::Enter => {
                if let Some(idx) = app.items_state.selected() {
                    app.current_article = Some(app.items[idx].content.clone());
                    app.route = Route::ArticleView;
                }
            }
            KeyCode::Char('h') | KeyCode::Left => app.route = Route::FeedsList,
            KeyCode::Char('q') => app.quit = true,
            _ => {}
        },
        Route::ArticleView => match key.code {
            KeyCode::Char('h') | KeyCode::Left => app.route = Route::ItemsList,
            KeyCode::Char('q') => app.quit = true,
            _ => {}
        },
    }
}

fn move_selection_down_table(state: &mut TableState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => (i + 1) % len,
        None => 0,
    };
    state.select(Some(i));
}

fn move_selection_up_table(state: &mut TableState) {
    let i = match state.selected() {
        Some(i) if i > 0 => i - 1,
        _ => 0,
    };
    state.select(Some(i));
}

fn move_selection_down_list(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => (i + 1) % len,
        None => 0,
    };
    state.select(Some(i));
}

fn move_selection_up_list(state: &mut ListState) {
    let i = match state.selected() {
        Some(i) if i > 0 => i - 1,
        _ => 0,
    };
    state.select(Some(i));
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
