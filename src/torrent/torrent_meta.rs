use derive_more::Display;
use hightorrent::{MagnetLink, TorrentFile};
use serde::{Deserialize, Serialize};

use crate::torrent::download_meta::{DownloadMeta, DownloadMetaProvider};

#[derive(Clone, Display, Serialize, Deserialize)]
#[display(fmt = "{{ indexer_id: {}, guid: {} }}", indexer_id, guid)]
pub struct TorrentMeta {
    pub guid: String,
    pub indexer_id: u8,
    pub download_url: Option<String>,
    pub magnet_url: Option<String>,
}

impl TorrentMeta {
    pub async fn get_torrent_hash(&self, download_meta_provider: &impl DownloadMetaProvider) -> Result<String, String> {
        if self.magnet_url.is_some() {
            Ok(MagnetLink::new(self.magnet_url.as_ref().unwrap())
                .map_err(|err| err.to_string())?
                .hash()
                .to_string())
        } else if self.download_url.is_some() {
            match download_meta_provider.get_download_meta(self.download_url.as_ref().unwrap()).await {
                Ok(content) => {
                    match content {
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
                }
                Err(err) => Err(format!("Error when interacting with Prowlarr: {}", err)),
            }
        } else {
            Err(format!("Neither magnet nor download link exist for torrent {}", self))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use mockall::{predicate::*};

    use crate::torrent::download_meta::{DownloadMeta, MockDownloadMetaProvider};
    use crate::torrent::torrent_meta::TorrentMeta;

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
}
