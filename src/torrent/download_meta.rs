use async_trait::async_trait;
use bytes::Bytes;
#[cfg(test)]
use mockall::automock;

pub enum DownloadMeta {
    MagnetLink(String),
    TorrentFile(Bytes),
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait DownloadMetaProvider {
    async fn get_download_meta(&self, download_url: &str) -> Result<DownloadMeta, String>;
}
