use crate::uuid_mapper::UuidMapper;

#[derive(Clone)]
pub struct TorrentData {
    pub guid: String,
    pub indexer_id: u8,
    pub download_url: Option<String>,
    pub magnet_url: Option<String>,
}

pub type TorrentDataStore = UuidMapper<TorrentData>;
