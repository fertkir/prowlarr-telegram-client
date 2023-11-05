use derive_more::Display;

use crate::uuid_mapper::UuidMapper;

#[derive(Clone, Display)]
#[display(fmt = "{{ indexer_id: {}, guid: {} }}", indexer_id, guid)]
pub struct TorrentMeta {
    pub guid: String,
    pub indexer_id: u8,
    pub download_url: Option<String>,
    pub magnet_url: Option<String>,
}

pub type TorrentMetaStore = UuidMapper<TorrentMeta>;
