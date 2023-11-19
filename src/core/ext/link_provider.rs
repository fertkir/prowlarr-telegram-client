use async_trait::async_trait;
use bytes::Bytes;

pub enum Link {
    MagnetLink(String),
    TorrentFile(Bytes)
}

pub enum LinkError {
    LinkProviderError(String)
}

pub type LinkResult = Result<Link, LinkError>;

#[async_trait]
pub trait LinkProvider<T> {
    async fn get_link(&self, meta: &T) -> LinkResult;
}
