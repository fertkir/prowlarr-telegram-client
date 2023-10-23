use chrono::{DateTime, Utc};
use reqwest::{Client, Response};
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use crate::util;

pub struct ProwlarrClient {
    api_key: String,
    base_url: String,
    client: Client,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub guid: String,
    pub indexer_id: u8,
    pub title: String,
    pub size: u128,
    pub publish_date: DateTime<Utc>,
    pub download_url: Option<String>,
    pub magnet_url: Option<String>,
    pub info_url: String,
    pub seeders: u32,
    pub leechers: u32,
    pub grabs: Option<u32>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadParams {
    pub guid: String,
    pub indexer_id: u8,
}

impl ProwlarrClient {
    pub fn from_env() -> ProwlarrClient {
        ProwlarrClient {
            api_key: util::get_env("PROWLARR_API_KEY"),
            base_url: util::get_env("PROWLARR_BASE_URL"),
            client: Client::new(),
        }
    }

    pub async fn search(&self, query: &str) -> reqwest::Result<Vec<SearchResult>> {
        self.client.get(format!("{}/api/v1/search?apikey={}&query={}",
                                self.base_url, self.api_key, query))
            .send()
            .await?
            .json::<Vec<SearchResult>>()
            .await
    }

    pub async fn download(&self, params: &DownloadParams) -> reqwest::Result<Response> {
        self.client.post(format!("{}/api/v1/search?apikey={}", self.base_url, self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(params)
            .send()
            .await
    }
}
