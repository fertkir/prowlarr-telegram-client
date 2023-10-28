use std::fmt::{Display, Formatter};
use crate::uuid_mapper::UuidMapper;

#[derive(Clone)]
pub struct TorrentData {
    pub guid: String,
    pub indexer_id: u8,
    pub download_url: Option<String>,
    pub magnet_url: Option<String>,
}

impl Display for TorrentData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ indexerId: {}, guid: {} }}", self.indexer_id, self.guid)
    }
}

pub type TorrentDataStore = UuidMapper<TorrentData>;
