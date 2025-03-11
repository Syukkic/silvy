use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct Feed {
    pub rssurl: String,
    pub url: String,
    pub title: String,
    pub unread_count: u32,
}

#[derive(Debug, FromRow, Clone)]
pub struct Item {
    pub id: u32,
    pub guid: String,
    pub title: String,
    pub author: String,
    pub url: String,
    pub feedurl: String,
    pub pub_date: String,
    pub content: String,
    // 1: true, 0: false
    pub unread: u8,
}
