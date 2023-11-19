use async_trait::async_trait;

pub type DownloadResult = Result<TorrentHash, DownloadError>;

pub type TorrentHash = String;

pub enum DownloadError {
    DownloadProviderError(String),
}

#[async_trait]
pub trait Downloader<T> {
    async fn download(&self, meta: &T) -> DownloadResult;
}
