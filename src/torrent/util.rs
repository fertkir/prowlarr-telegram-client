use hightorrent::{MagnetLink, TorrentFile};

use crate::torrent::download_meta::{DownloadMeta, DownloadMetaProvider};
use crate::torrent::torrent_meta::TorrentMeta;

pub async fn get_torrent_hash(torrent_data: &TorrentMeta,
                              download_meta_provider: &impl DownloadMetaProvider) -> Result<String, String> {
    if torrent_data.magnet_url.is_some() {
        Ok(MagnetLink::new(torrent_data.magnet_url.as_ref().unwrap())
            .map_err(|err| err.to_string())?
            .hash()
            .to_string())
    } else if torrent_data.download_url.is_some() {
        match download_meta_provider.get_download_meta(torrent_data.download_url.as_ref().unwrap()).await {
            Ok(content) => {
                match content {
                    DownloadMeta::MagnetLink(link) =>
                        Ok(MagnetLink::new(&link)
                            .map_err(|err| err.to_string())? // todo add info about where an error occurred
                            .hash()
                            .to_string()), // fixme: https://github.com/angrynode/hightorrent/issues/2
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
        Err(format!("Neither magnet nor download link exist for torrent {}", torrent_data))
    }
}
