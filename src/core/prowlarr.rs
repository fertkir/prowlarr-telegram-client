use std::{env, fs};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::{CONTENT_TYPE, LOCATION};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::core::download_meta::{DownloadMeta, DownloadMetaProvider};

pub struct ProwlarrClient {
    api_key: String,
    base_url: Url,
    limit_param: String,
    indexer_id_params: String,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadParams<'a> {
    guid: &'a str,
    indexer_id: &'a u8,
}

const PROWLARR_API_KEY_ENV: &str = "PROWLARR_API_KEY";
const PROWLARR_API_KEY_FILE_ENV: &str = "PROWLARR_API_KEY_FILE";
const PROWLARR_BASE_URL_ENV: &str = "PROWLARR_BASE_URL";
const PROWLARR_DEFAULT_LIMIT_PARAM_ENV: &str = "PROWLARR_DEFAULT_LIMIT_PARAM";
const PROWLARR_INDEXER_IDS_ENV: &str = "PROWLARR_INDEXER_IDS";

impl ProwlarrClient {
    pub fn from_env() -> ProwlarrClient {
        ProwlarrClient {
            api_key: get_api_key(),
            base_url: ProwlarrClient::parse_base_url(),
            limit_param: ProwlarrClient::get_limit_param(),
            indexer_id_params: ProwlarrClient::get_indexer_id_params(),
            client: Client::new(),
        }
    }

    fn get_limit_param() -> String {
        match env::var(PROWLARR_DEFAULT_LIMIT_PARAM_ENV) {
            Ok(val) => {
                let n: u32 = val
                    .parse()
                    .unwrap_or_else(|_| panic!("{} must be a non-negative number", PROWLARR_DEFAULT_LIMIT_PARAM_ENV));
                format!("&limit={}", n)
            }
            Err(_) => String::new(),
        }
    }

    fn parse_base_url() -> Url {
        let url_string = get_env(PROWLARR_BASE_URL_ENV);
        Url::parse(&url_string)
            .unwrap_or_else(|err|
                panic!("Could not parse {}: {}: \"{}\"", PROWLARR_BASE_URL_ENV, err, url_string))
    }

    fn get_indexer_id_params() -> String {
        let params = env::var(PROWLARR_INDEXER_IDS_ENV)
            .unwrap_or_default()
            .split(',')
            .filter(|indexer_id| !indexer_id.is_empty())
            .map(|user| user.parse::<u32>()
                .unwrap_or_else(|_| panic!("{} list must be a comma-separated \
                string of integers. Value \"{}\" is unexpected", PROWLARR_INDEXER_IDS_ENV, user)))
            .map(|n| format!("indexerIds={}", n))
            .collect::<Vec<String>>()
            .join("&");
        if params.is_empty() {
            params
        } else {
            format!("&{}", params)
        }
    }

    pub async fn search(&self, query: &str) -> reqwest::Result<Vec<SearchResult>> {
        self.client.get(format!("{}api/v1/search?apikey={}{}&query={}{}", self.base_url,
                                self.api_key, self.limit_param, query, self.indexer_id_params))
            .send()
            .await?
            .json::<Vec<SearchResult>>()
            .await
    }

    pub async fn download(&self, indexer_id: &u8, guid: &str) -> reqwest::Result<Response> {
        self.client.post(format!("{}api/v1/search?apikey={}", self.base_url, self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&DownloadParams { guid, indexer_id })
            .send()
            .await
    }
}

#[async_trait]
impl DownloadMetaProvider for ProwlarrClient {
    async fn get_download_meta(&self, download_url: &str) -> Result<DownloadMeta, String> {
        let response = self.client.get(replace_base_url(download_url, &self.base_url)?)
            .send()
            .await
            .map_err(|err| err.to_string())?;
        if response.status().is_redirection() {
            let magnet = response.headers().get(LOCATION)
                .ok_or("Missing expected Location header")?
                .to_str()
                .map_err(|err| err.to_string())?
                .to_string();
            Ok(DownloadMeta::MagnetLink(magnet))
        } else if response.status().is_success() {
            let torrent_file = response.bytes()
                .await
                .map_err(|err| err.to_string())?;
            Ok(DownloadMeta::TorrentFile(torrent_file))
        } else {
            Err(format!("Unexpected response status code: {}", response.status()))
        }
    }
}

fn replace_base_url(url: &str, base_url: &Url) -> Result<String, String> {
    let mut url = Url::parse(url).map_err(|err| err.to_string())?;
    url.set_host(base_url.host_str()).unwrap();
    url.set_port(base_url.port()).unwrap();
    Ok(url.to_string())
}

fn get_env(env: &str) -> String {
    env::var(env).unwrap_or_else(|_| panic!("Cannot get the {env} env variable"))
}

fn get_api_key() -> String {
    if let Ok(api_key) = env::var(PROWLARR_API_KEY_ENV) {
        api_key
    } else if let Ok(api_key_file) = env::var(PROWLARR_API_KEY_FILE_ENV) {
        fs::read_to_string(api_key_file.clone())
            .unwrap_or_else(|_| panic!("Could not read {PROWLARR_API_KEY_FILE_ENV} file {api_key_file}"))
    } else {
        panic!("Neither {PROWLARR_API_KEY_ENV} nor {PROWLARR_API_KEY_FILE_ENV} env variable is provided")
    }
}

#[cfg(test)]
mod test {
    mod get_api_key {
        use crate::core::prowlarr::{get_api_key, PROWLARR_API_KEY_ENV, PROWLARR_API_KEY_FILE_ENV};
        use std::fs::File;
        use std::io::Write;

        #[test]
        fn from_env_var() {
            temp_env::with_var(PROWLARR_API_KEY_ENV, Some("key"), || {
                assert_eq!(get_api_key(), "key")
            });
        }

        #[test]
        fn from_file() {
            let dir = tempfile::tempdir().unwrap();
            let file_path = dir.path().join("file_with_key");
            let file_path_str = file_path.to_str().unwrap().to_string();
            let mut file = File::create(file_path).unwrap();
            write!(file, "key").unwrap();
            temp_env::with_var(PROWLARR_API_KEY_FILE_ENV, Some(file_path_str), || {
                assert_eq!(get_api_key(), "key")
            });
        }

        #[test]
        #[should_panic(expected = "Could not read PROWLARR_API_KEY_FILE file /unknown/path")]
        fn panic_if_no_such_file() {
            temp_env::with_var(PROWLARR_API_KEY_FILE_ENV, Some("/unknown/path"), || {
                get_api_key()
            });
        }

        #[test]
        #[should_panic(expected = "Neither PROWLARR_API_KEY nor PROWLARR_API_KEY_FILE env variable is provided")]
        fn panic_if_no_vars_provided() {
            temp_env::with_var_unset(PROWLARR_API_KEY_FILE_ENV, || {
                get_api_key()
            });
        }
    }

    mod get_env {
        use crate::core::prowlarr::get_env;

        #[test]
        fn get_var() {
            temp_env::with_var("SOME_VAR", Some("some value"), || {
                assert_eq!(get_env("SOME_VAR"), "some value")
            });
        }

        #[test]
        #[should_panic(expected = "Cannot get the MISSING_VAR env variable")]
        fn panic_if_no_var() {
            get_env("MISSING_VAR");
        }
    }

    mod client {
        use crate::core::prowlarr::{ProwlarrClient, PROWLARR_API_KEY_ENV, PROWLARR_BASE_URL_ENV, PROWLARR_DEFAULT_LIMIT_PARAM_ENV};
        use chrono::DateTime;
        use reqwest::header::CONTENT_TYPE;
        use reqwest::StatusCode;
        use wiremock::matchers::{header, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[test]
        #[should_panic(expected = "Could not parse PROWLARR_BASE_URL: relative URL without a base: \"incorrect_url\"")]
        fn bad_base_url() {
            temp_env::with_vars([(PROWLARR_API_KEY_ENV, Some("key")),
                                    (PROWLARR_BASE_URL_ENV, Some("incorrect_url"))], || {
                ProwlarrClient::from_env()
            });
        }

        #[tokio::test]
        async fn search() {
            let mock_server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/api/v1/search"))
                .and(query_param("apikey", "key123"))
                .and(query_param("query", "Ubuntu"))
                .and(query_param("limit", "100"))
                .respond_with(ResponseTemplate::new(200)
                    .set_body_string(
                        "[{\"guid\":\"101\",\"indexerId\":1,\"title\":\"Title\",\
                        \"size\":20000,\"publishDate\":\"2015-05-15T00:00:00Z\",\
                        \"infoUrl\":\"info url\",\"downloadUrl\":\"download url\",\
                        \"magnetUrl\":\"magnet url\",\"seeders\":10,\"leechers\":20,\"grabs\":5}]"))
                .mount(&mock_server)
                .await;

            let prowlarr_client = temp_env::with_vars(
                [(PROWLARR_API_KEY_ENV, Some("key123")),
                    (PROWLARR_DEFAULT_LIMIT_PARAM_ENV, Some("100")),
                    (PROWLARR_BASE_URL_ENV, Some(&mock_server.uri()))],
                ProwlarrClient::from_env);

            let result = prowlarr_client.search("Ubuntu").await.unwrap();

            assert_eq!(result.len(), 1);

            let search_result = result.first().unwrap();
            assert_eq!(search_result.guid, "101");
            assert_eq!(search_result.indexer_id, 1);
            assert_eq!(search_result.title, "Title");
            assert_eq!(search_result.size, 20000);
            assert_eq!(search_result.publish_date,
                       DateTime::from_timestamp(1431648000, 0).unwrap());
            assert_eq!(search_result.download_url, Some("download url".to_string()));
            assert_eq!(search_result.magnet_url, Some("magnet url".to_string()));
            assert_eq!(search_result.info_url, "info url");
            assert_eq!(search_result.seeders, 10);
            assert_eq!(search_result.leechers, 20);
            assert_eq!(search_result.grabs, Some(5));
        }

        #[tokio::test]
        async fn download() {
            let mock_server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(header(CONTENT_TYPE.as_str(), "application/json"))
                .and(path("/api/v1/search"))
                .and(query_param("apikey", "key123"))
                .respond_with(ResponseTemplate::new(200))
                .mount(&mock_server)
                .await;

            let prowlarr_client = temp_env::with_vars(
                [(PROWLARR_API_KEY_ENV, Some("key123")),
                    (PROWLARR_BASE_URL_ENV, Some(&mock_server.uri()))],
                ProwlarrClient::from_env);

            let result = prowlarr_client.download(&1, "guid123").await.unwrap();

            assert_eq!(result.status(), StatusCode::OK);
        }

        mod download_url_content {
            use crate::core::prowlarr::{ProwlarrClient, PROWLARR_API_KEY_ENV, PROWLARR_BASE_URL_ENV};
            use reqwest::header::LOCATION;
            use wiremock::matchers::{method, path};
            use wiremock::{Mock, MockServer, ResponseTemplate};

            use crate::core::download_meta::{DownloadMeta, DownloadMetaProvider};

            const DOWNLOAD_URL: &str = "http://localhost:9696/content";

            #[tokio::test]
            async fn magnet_link() {
                let mock_server = MockServer::start().await;
                let magnet_link = "magnet:?xt=urn:btih:63A46761289B3D1";
                Mock::given(method("GET"))
                    .and(path("/content"))
                    .respond_with(ResponseTemplate::new(302)
                        .append_header(LOCATION.as_str(), magnet_link))
                    .mount(&mock_server)
                    .await;

                let prowlarr_client = temp_env::with_vars(
                    [(PROWLARR_API_KEY_ENV, Some("key123")),
                        (PROWLARR_BASE_URL_ENV, Some(&mock_server.uri()))],
                    ProwlarrClient::from_env);

                let result = prowlarr_client.get_download_meta(DOWNLOAD_URL).await.unwrap();
                match result {
                    DownloadMeta::MagnetLink(link) => assert_eq!(link, magnet_link.to_string()),
                    DownloadMeta::TorrentFile(_) => panic!("torrent file unexpected")
                }
            }

            #[tokio::test]
            async fn torrent_file() {
                let mock_server = MockServer::start().await;
                Mock::given(method("GET"))
                    .and(path("/content"))
                    .respond_with(ResponseTemplate::new(200)
                        .set_body_string("file contents"))
                    .mount(&mock_server)
                    .await;

                let prowlarr_client = temp_env::with_vars(
                    [(PROWLARR_API_KEY_ENV, Some("key123")),
                        (PROWLARR_BASE_URL_ENV, Some(&mock_server.uri()))],
                    ProwlarrClient::from_env);

                let result = prowlarr_client.get_download_meta(DOWNLOAD_URL).await.unwrap();
                match result {
                    DownloadMeta::MagnetLink(_) => panic!("magnet link unexpected"),
                    DownloadMeta::TorrentFile(file) => assert_eq!(file, "file contents")
                }
            }

            #[tokio::test]
            #[should_panic(expected = "Unexpected response status code: 404")]
            async fn not_found() {
                let mock_server = MockServer::start().await;
                Mock::given(method("GET"))
                    .and(path("/content"))
                    .respond_with(ResponseTemplate::new(404))
                    .mount(&mock_server)
                    .await;

                let prowlarr_client = temp_env::with_vars(
                    [(PROWLARR_API_KEY_ENV, Some("key123")),
                        (PROWLARR_BASE_URL_ENV, Some(&mock_server.uri()))],
                    ProwlarrClient::from_env);

                prowlarr_client.get_download_meta(DOWNLOAD_URL).await.unwrap();
            }
        }
    }
}
