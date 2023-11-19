use chrono::{DateTime, Utc};

pub struct SearchResponse {
    pub title: String,
    pub size: u128,
    pub publish_date: DateTime<Utc>,
    pub info_url: String,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: Option<u32>,
}

pub trait SearchResponseFormatter {
    fn format(&self, search_response: &SearchResponse, uuid: &String, locale: &str) -> String;
}
