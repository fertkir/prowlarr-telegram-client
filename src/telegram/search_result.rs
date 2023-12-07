use byte_unit::Byte;
use byte_unit::UnitType::Decimal;
use teloxide::utils::markdown::{bold, escape, link};

use crate::core::prowlarr::SearchResult;
use crate::torrent::torrent_meta::TorrentMeta;

impl SearchResult {
    pub fn to_message(&self, bot_uuid: &str, locale: &str) -> String {
        format!("{}\n{}\nS {} \\| L {} \\| {} \\| {} {} \\| {} {}\n{}: /d\\_{}\n{}: /m\\_{}\n\n",
                escape(&self.title),
                link(&self.info_url, &t!("description", locale = &locale)),
                self.seeders, self.leechers, self.downloads(locale), &t!("registered", locale = &locale),
                escape(&self.publish_date.date_naive().to_string()),
                &t!("size", locale = &locale), &self.size(),
                bold(&t!("download", locale = &locale)), bot_uuid,
                escape(&t!("get_link", locale = &locale)), bot_uuid)
    }

    fn downloads(&self, locale: &str) -> String {
        self.grabs
            .map(|grabs| format!("{} {}", t!("downloaded", locale = &locale), grabs))
            .unwrap_or_default()
    }

    fn size(&self) -> String {
        let size = Byte::from_u128(self.size)
            .map(|b| b.get_appropriate_unit(Decimal))
            .map(|b| format!("{b:#.2}"))
            .unwrap_or_else(|| "???".to_string());
        escape(&size)
    }
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

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::core::prowlarr::SearchResult;
    use crate::torrent::torrent_meta::TorrentMeta;

    #[test]
    fn search_result_to_message() {
        let search_result = SearchResult {
            guid: "ubuntu_22_04".to_string(),
            indexer_id: 2,
            title: "Ubuntu 22.04".to_string(),
            size: 1234567,
            publish_date: DateTime::from_timestamp(1431648000, 0).unwrap(),
            download_url: None,
            magnet_url: None,
            info_url: "http://localhost/ubuntu".to_string(),
            seeders: 20,
            leechers: 10,
            grabs: Some(10000),
        };
        let bot_uuid = "uuid";

        let result = search_result.to_message(bot_uuid, "en");

        assert_eq!(result, "Ubuntu 22\\.04\n\
            [Description](http://localhost/ubuntu)\n\
            S 20 \\| L 10 \\| Downloaded 10000 \\| Reg 2015\\-05\\-15 \\| Size 1\\.23 MB\n\
            *Download*: /d\\_uuid\n\
            Get link/torrent\\-file: /m\\_uuid\n\n")
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
