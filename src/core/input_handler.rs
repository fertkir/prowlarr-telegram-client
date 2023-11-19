use crate::core::downloads_tracker::DownloadsTracker;
use crate::core::ext::downloader::{Downloader, DownloadError};
use crate::core::ext::error::HandlingResult;
use crate::core::ext::input::{Command, Destination, Input, ItemUuid, Locale, SearchQuery, Source};
use crate::core::ext::link_provider::{Link, LinkError, LinkProvider};
use crate::core::ext::search_response::{SearchResponse, SearchResponseFormatter};
use crate::core::ext::searcher::{Searcher, SearchError};
use crate::core::ext::sender::Sender;
use crate::core::ext::uuid_mapper::UuidMapper;
use crate::core::permissions::PermissionChecker;

pub struct InputHandler<T> {
    pub permission_checker: PermissionChecker,
    pub searcher: Box<dyn Searcher>,
    pub downloader: Box<dyn Downloader<T>>,
    pub link_provider: Box<dyn LinkProvider<T>>,
    pub uuid_mapper: UuidMapper<T>,
    pub sender: Box<dyn Sender>,
    pub search_response_formatter: Box<dyn SearchResponseFormatter>,
    pub downloads_tracker: DownloadsTracker,
}

const RESULTS_COUNT: usize = 10;

impl<T> InputHandler<T> {
    pub async fn handle(&self, input: Box<dyn Input>) -> HandlingResult {
        let source = input.get_source();
        let destination = input.get_destination();
        let locale = input.get_locale();
        if self.permission_checker.is_allowed(&source) {
            match input.get_command() {
                Command::Search(query) => self.search(source, &destination, &locale, &query).await?,
                Command::Download(uuid) => self.download(&destination, &locale, &uuid).await?,
                Command::GetLink(uuid) => self.link(source, &destination, &locale, &uuid).await?,
                Command::Help => self.sender.send_plain_message(&destination, &t!("help", locale = &locale)).await?,
                Command::Ignore => {
                    // do nothing
                }
            }
        }
        Ok(())
    }

    async fn search(&self, source: Source, destination: &Destination, locale: &Locale, query: &SearchQuery) -> HandlingResult {
        log::info!("from {} | Received search request \"{}\"", source, query);
        match self.searcher.search(&query).await {
            Ok(response) => {
                let response = sorted_by_seeders(response)
                    .iter()
                    .take(RESULTS_COUNT)
                    .map(|search_result| {
                        let bot_uuid = self.uuid_mapper.put(search_result.into());
                        self.search_response_formatter.format(&search_result, &bot_uuid, &locale)
                    })
                    .reduce(|acc, e| acc + &e);
                match response {
                    None => {
                        self.sender.send_plain_message(destination, &t!("no_results", locale = &locale, request = query)).await?;
                        log::info!("  to {} | Sent \"No results\" response", destination);
                    }
                    Some(response) => {
                        self.sender.send_message(destination, &response).await?;
                        log::info!("  to {} | Sent search response \"{}\"", destination, to_digest(&response));
                    }
                }
            }
            Err(err) => match err {
                SearchError::SearchProviderError(err) => {
                    log::error!("  to {} | Error interacting with search provider: {}", destination, err);
                    self.sender.send_message(destination, &t!("search_provider_error", locale = &locale)).await
                }
            }
        }
        Ok(())
    }

    async fn download(&self, destination: &Destination, locale: &Locale, uuid: &ItemUuid) -> HandlingResult {
        match self.uuid_mapper.get(&uuid) {
            None => self.link_not_found(destination, &locale, &uuid).await?,
            Some(meta) => {
                match self.downloader.download(&meta).await {
                    Ok(hash) => {
                        self.downloads_tracker.add(hash, destination, locale)
                    }
                    Err(err) => match err {
                        DownloadError::DownloadProviderError(err) => {
                            log::error!("  to {} | Error interacting with download provider: {}", destination, err);
                            self.sender.send_message(destination, &t!("download_provider_error", locale = &locale)).await
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn link(&self, source: Source, destination: &Destination, locale: &Locale, uuid: &ItemUuid) -> HandlingResult {
        log::info!("from {} | Received download request for {}", source, uuid);
        match self.uuid_mapper.get(&uuid) {
            None => self.link_not_found(destination, &locale, uuid).await?,
            Some(meta) => {
                match self.link_provider.get_link(&meta).await {
                    Ok(link) => match link {
                        Link::MagnetLink(link) => {
                            self.sender.send_magnet(destination, &link).await?;
                            log::info!("  to {} | Sent magnet link for {} ", destination, &meta);
                        }
                        Link::TorrentFile(file) => {
                            self.sender.send_torrent_file(destination, &format!("{}.torrent", uuid), file).await?;
                            log::info!("userId {} | Sent .torrent file for {} ", destination, &meta);
                        }
                    }
                    Err(err) => match err {
                        LinkError::LinkProviderError(err) => {
                            log::error!("  to {} | Error interacting with link provider: {}", destination, err);
                            self.sender.send_message(destination, &t!("link_provider_error", locale = &locale)).await
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn link_not_found(&self, destination: &Destination, locale: &Locale, uuid: &ItemUuid) -> HandlingResult {
        log::warn!("  to {} | Link for uuid {} not found", destination, &uuid);
        self.sender.send_message(destination, &t!("link_not_found", locale = locale)).await?;
        Ok(())
    }
}

fn sorted_by_seeders(mut results: Vec<SearchResponse>) -> Vec<SearchResponse> {
    results.sort_unstable_by(|a, b| b.seeders.cmp(&a.seeders));
    results
}

fn to_digest(str: &str) -> String {
    str.char_indices()
        .map(|(i, _)| i)
        .nth(100)
        .map(|end| str[0..end].to_string())
        .unwrap_or(str.to_string())
}
