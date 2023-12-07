use byte_unit::Byte;
use byte_unit::UnitType::Decimal;
use teloxide::utils::markdown::{bold, escape, link};

use crate::core::prowlarr::SearchResult;
use crate::core::traits::search_result_serializer::SearchResultSerializer;

pub struct TgSearchResultSerializer;

impl SearchResultSerializer for TgSearchResultSerializer {
    fn serialize(&self, search_result: &SearchResult, bot_uuid: &str, locale: &str) -> String {
        format!("{}\n{}\nS {} \\| L {} \\| {} \\| {} {} \\| {} {}\n{}: /d\\_{}\n{}: /m\\_{}\n\n",
                escape(&search_result.title),
                link(&search_result.info_url, &t!("description", locale = &locale)),
                search_result.seeders, search_result.leechers, downloads(&search_result, locale), &t!("registered", locale = &locale),
                escape(&search_result.publish_date.date_naive().to_string()),
                &t!("size", locale = &locale), size(&search_result),
                bold(&t!("download", locale = &locale)), bot_uuid,
                escape(&t!("get_link", locale = &locale)), bot_uuid)
    }
}

fn downloads(search_result: &SearchResult, locale: &str) -> String {
    search_result.grabs
        .map(|grabs| format!("{} {}", t!("downloaded", locale = &locale), grabs))
        .unwrap_or_default()
}

fn size(search_result: &SearchResult) -> String {
    let size = Byte::from_u128(search_result.size)
        .map(|b| b.get_appropriate_unit(Decimal))
        .map(|b| format!("{b:#.2}"))
        .unwrap_or_else(|| "???".to_string());
    escape(&size)
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::core::prowlarr::SearchResult;
    use crate::core::traits::search_result_serializer::SearchResultSerializer;
    use crate::ext::search_result_serializer::telegram::TgSearchResultSerializer;

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

        let result = TgSearchResultSerializer.serialize(&search_result, bot_uuid, "en");

        assert_eq!(result, "Ubuntu 22\\.04\n\
            [Description](http://localhost/ubuntu)\n\
            S 20 \\| L 10 \\| Downloaded 10000 \\| Reg 2015\\-05\\-15 \\| Size 1\\.23 MB\n\
            *Download*: /d\\_uuid\n\
            Get link/torrent\\-file: /m\\_uuid\n\n")
    }
}
