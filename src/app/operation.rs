use super::{App, Route};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::{ListState, TableState};

pub fn handle_key_events(app: &mut App, key: KeyEvent) {
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
            KeyCode::Char('j') | KeyCode::Down => app.article_scroll += 1,
            KeyCode::Char('k') | KeyCode::Up => {
                if app.article_scroll > 0 {
                    app.article_scroll -= 1;
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.route = Route::ItemsList;
                app.article_scroll = 0;
            }
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
