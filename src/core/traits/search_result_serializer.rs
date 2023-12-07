use crate::core::prowlarr::SearchResult;

pub trait SearchResultSerializer: Send + Sync {
    fn serialize(&self, search_result: &SearchResult, bot_uuid: &str, locale: &str) -> String;
}
