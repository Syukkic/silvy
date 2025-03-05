use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Feed {
    pub rssurl: String,
    pub url: String,
    pub title: String,
}

impl Feed {
    pub fn new(rssurl: String, url: String, title: String) -> Self {
        Self { rssurl, url, title }
    }
}

#[derive(Debug, FromRow)]
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

impl Item {
    pub fn new(
        id: u32,
        guid: String,
        title: String,
        author: String,
        url: String,
        feedurl: String,
        pub_date: String,
        content: String,
        unread: u8,
    ) -> Self {
        Self {
            id,
            guid,
            title,
            author,
            url,
            feedurl,
            pub_date,
            content,
            unread,
        }
    }
}
