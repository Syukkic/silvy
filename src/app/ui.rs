use super::{App, Route};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Wrap},
};

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
        .wrap(Wrap { trim: true })
        .scroll((app.article_scroll as u16, 0));

    frame.render_widget(paragraph, area);
}
