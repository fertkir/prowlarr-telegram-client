use derive_more::Display;
use hightorrent::{MagnetLink, TorrentFile};
use serde::{Deserialize, Serialize};

use crate::core::prowlarr::SearchResult;
use crate::core::download_meta::{DownloadMeta, DownloadMetaProvider};

#[derive(Clone, Display, Serialize, Deserialize)]
#[display("{{ indexer_id: {}, guid: {} }}", indexer_id, guid)]
pub struct TorrentMeta {
    pub guid: String,
    pub indexer_id: u8,
    pub download_url: Option<String>,
    pub magnet_url: Option<String>,
}

impl From<&SearchResult> for TorrentMeta {
    fn from(value: &SearchResult) -> Self {
        TorrentMeta {
            indexer_id: value.indexer_id,
            download_url: value.download_url.clone(),
            guid: value.guid.clone(),
            magnet_url: value.magnet_url.clone(),
        }
    }
}

impl TorrentMeta {
    pub async fn get_torrent_hash(&self, download_meta_provider: &impl DownloadMetaProvider) -> Result<String, String> {
        if let Some(magnet_hash) = self.magnet_url.as_ref()
            .and_then(|url| MagnetLink::new(url).ok())
            .map(|magnet| magnet.hash().to_string()) {
            return Ok(magnet_hash);
        }
        let url = self.magnet_url.as_ref()
            .or(self.download_url.as_ref());
        if url.is_none() {
            return Err(format!("Neither magnet nor download link exist for torrent {}", self));
        }
        match download_meta_provider.get_download_meta(url.unwrap()).await {
            Ok(content) => match content {
                DownloadMeta::MagnetLink(link) =>
                    Ok(MagnetLink::new(&link)
                        .map_err(|err| err.to_string())? // todo add info about where an error occurred
                        .hash()
                        .to_string()),
                DownloadMeta::TorrentFile(torrent_file) =>
                    Ok(TorrentFile::from_slice(torrent_file.as_ref())
                        .map_err(|err| err.to_string())?
                        .hash()
                        .to_string()),
            }
            Err(err) => Err(format!("Error when interacting with Prowlarr: {}", err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use mockall::{predicate::*};

    use crate::core::prowlarr::SearchResult;
    use crate::core::download_meta::{DownloadMeta, MockDownloadMetaProvider};
    use crate::core::torrent_meta::TorrentMeta;

    #[tokio::test]
    async fn magnet_link_from_torrent_meta() {
        let meta_provider = MockDownloadMetaProvider::new();
        let torrent_meta = TorrentMeta {
            guid: "".to_string(),
            indexer_id: 0,
            download_url: None,
            magnet_url: Some("magnet:?xt=urn:btih:c811b41641a09d192b8ed81b14064fff55d85ce3".to_string()),
        };

        let hash = torrent_meta.get_torrent_hash(&meta_provider)
            .await
            .unwrap();

        assert_eq!(hash, "c811b41641a09d192b8ed81b14064fff55d85ce3");
    }

    #[tokio::test]
    async fn magnet_link_from_provider() {
        let mut meta_provider = MockDownloadMetaProvider::new();
        meta_provider
            .expect_get_download_meta()
            .with(eq("download_url"))
            .times(1)
            .returning(move |_| Box::pin(async move {
                Ok(DownloadMeta::MagnetLink("magnet:?xt=urn:btih:c811b41641a09d192b8ed81b14064fff55d85ce3".to_string()))
            }));
        let torrent_meta = TorrentMeta {
            guid: "".to_string(),
            indexer_id: 0,
            download_url: Some("download_url".to_string()),
            magnet_url: None,
        };

        let hash = torrent_meta.get_torrent_hash(&meta_provider)
            .await
            .unwrap();

        assert_eq!(hash, "c811b41641a09d192b8ed81b14064fff55d85ce3");
    }

    #[tokio::test]
    async fn torrent_file_from_provider() {
        let mut meta_provider = MockDownloadMetaProvider::new();
        meta_provider
            .expect_get_download_meta()
            .with(eq("download_url"))
            .times(1)
            .returning(move |_| Box::pin(async move {
                Ok(DownloadMeta::TorrentFile(Bytes::from(std::fs::read("tests/debian.torrent").unwrap())))
            }));
        let torrent_meta = TorrentMeta {
            guid: "".to_string(),
            indexer_id: 0,
            download_url: Some("download_url".to_string()),
            magnet_url: None,
        };

        let hash = torrent_meta.get_torrent_hash(&meta_provider)
            .await
            .unwrap();

        assert_eq!(hash, "d55be2cd263efa84aeb9495333a4fabc428a4250");
    }

    #[test]
    fn search_result_to_torrent_meta() {
        let search_result = SearchResult {
            guid: "ubuntu_22_04".to_string(),
            indexer_id: 2,
            title: "".to_string(),
            size: 0,
            publish_date: Default::default(),
            download_url: Some("download".to_string()),
            magnet_url: Some("magnet".to_string()),
            info_url: "".to_string(),
            seeders: 0,
            leechers: 0,
            grabs: None,
        };

        let result: TorrentMeta = (&search_result).into();

        assert_eq!(result.guid, "ubuntu_22_04");
        assert_eq!(result.indexer_id, 2);
        assert_eq!(result.magnet_url, Some("magnet".to_string()));
        assert_eq!(result.download_url, Some("download".to_string()));
    }
}
