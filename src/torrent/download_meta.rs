use async_trait::async_trait;
use bytes::Bytes;

pub enum DownloadMeta {
    MagnetLink(String),
    TorrentFile(Bytes),
}

#[async_trait]
pub trait DownloadMetaProvider {
    async fn get_download_meta(&self, download_url: &str) -> Result<DownloadMeta, String>;
}
