use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone)]
pub struct ProwlarrClient {
    api_key: String,
    base_url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub guid: String,
    pub title: String,
    pub size: u128,
    pub publish_date: DateTime<Utc>,
    pub download_url: Option<String>,
    pub magnet_url: Option<String>,
    pub info_url: String,
    pub seeders: u32,
    pub leechers: u32,
    pub grabs: Option<u32>
}

impl ProwlarrClient {
    pub fn from_env() -> ProwlarrClient {
        ProwlarrClient {
            api_key: std::env::var("PROWLARR_API_KEY").unwrap(),
            base_url: std::env::var("PROWLARR_BASE_URL").unwrap(),
        }
    }

    pub async fn search(self, query: &str) -> reqwest::Result<Vec<SearchResult>> {
        let url = format!("{}/api/v1/search?apikey={}&query={}",
                             self.base_url, self.api_key, query);
        reqwest::get(url)
            .await?
            .json::<Vec<SearchResult>>()
            .await
    }
}
