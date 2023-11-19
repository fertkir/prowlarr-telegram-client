use async_trait::async_trait;

use crate::core::ext::search_response::SearchResponse;

pub enum SearchError {
    SearchProviderError(String)
}

pub type SearchResult = Result<Vec<SearchResponse>, SearchError>;

#[async_trait]
pub trait Searcher {
    async fn search(&self, query: &str) -> SearchResult;
}
