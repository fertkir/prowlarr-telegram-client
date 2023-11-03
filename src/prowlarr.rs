use bytes::Bytes;
use chrono::{DateTime, Utc};
use reqwest::{Client, Response};
use reqwest::header::{CONTENT_TYPE, LOCATION};
use serde::{Deserialize, Serialize};
use url::Url;

pub struct ProwlarrClient {
    api_key: String,
    base_url: Url,
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

pub enum DownloadUrlContent {
    MagnetLink(String),
    TorrentFile(Bytes)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadParams<'a> {
    guid: &'a str,
    indexer_id: &'a u8,
}

const PROWLARR_API_KEY_ENV: &str = "PROWLARR_API_KEY";
const PROWLARR_BASE_URL_ENV: &str = "PROWLARR_BASE_URL";

impl ProwlarrClient {
    pub fn from_env() -> ProwlarrClient {
        ProwlarrClient {
            api_key: get_env(PROWLARR_API_KEY_ENV), // todo fall back to a file with the key
            base_url: ProwlarrClient::parse_base_url(),
            client: Client::new(),
        }
    }

    fn parse_base_url() -> Url {
        let url_string = get_env(PROWLARR_BASE_URL_ENV);
        Url::parse(&url_string)
            .unwrap_or_else(|err|
                panic!("Could not parse {}: {}: \"{}\"", PROWLARR_BASE_URL_ENV, err, url_string))
    }

    pub async fn search(&self, query: &str) -> reqwest::Result<Vec<SearchResult>> {
        self.client.get(format!("{}api/v1/search?apikey={}&query={}",
                                self.base_url, self.api_key, query))
            .send()
            .await?
            .json::<Vec<SearchResult>>()
            .await
    }

    pub async fn download(&self, indexer_id: &u8, guid: &str) -> reqwest::Result<Response> {
        self.client.post(format!("{}api/v1/search?apikey={}", self.base_url, self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&DownloadParams{ guid, indexer_id })
            .send()
            .await
    }

    pub async fn get_download_url_content(&self, download_url: &str) -> Result<DownloadUrlContent, String> {
        let response = self.client.get(self.replace_base_url(download_url)?)
            .send()
            .await
            .map_err(|err|err.to_string())?;
        if response.status().is_redirection() {
            let magnet = response.headers().get(LOCATION)
                .ok_or("Missing expected Location header")?
                .to_str()
                .map_err(|err|err.to_string())?
                .to_string();
            Ok(DownloadUrlContent::MagnetLink(magnet))
        } else if response.status().is_success() {
            let torrent_file = response.bytes()
                .await
                .map_err(|err| err.to_string())?;
            Ok(DownloadUrlContent::TorrentFile(torrent_file))
        } else {
            Err(format!("Unexpected response status code: {}", response.status()))
        }
    }

    fn replace_base_url(&self, url: &str) -> Result<String, String> {
        let mut url = Url::parse(url).map_err(|err|err.to_string())?;
        url.set_host(self.base_url.host_str()).unwrap();
        url.set_port(self.base_url.port()).unwrap();
        Ok(url.to_string())
    }
}

fn get_env(env: &str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get the {env} env variable"))
}
