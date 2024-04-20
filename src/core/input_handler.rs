use std::fmt::Display;
use std::sync::Arc;

use crate::core::download_meta::{DownloadMeta, DownloadMetaProvider};
use crate::core::downloads_tracker::DownloadsTracker;
use crate::core::HandlingResult;
use crate::core::prowlarr::{ProwlarrClient, SearchResult};
use crate::core::torrent_meta::TorrentMeta;
use crate::core::traits::input::{Command, Destination, Input, ItemUuid, Locale, ReplyToMessage, SearchQuery, Source};
use crate::core::traits::search_result_serializer::SearchResultSerializer;
use crate::core::traits::sender::Sender;
use crate::core::traits::uuid_mapper::{MapperError, UuidMapper};

pub struct InputHandler {
    prowlarr: ProwlarrClient,
    uuid_mapper: Box<dyn UuidMapper<TorrentMeta>>,
    downloads_tracker: Arc<DownloadsTracker>,
    allowed_users: Vec<u64>,
    sender: Box<dyn Sender>,
    search_result_serializer: Box<dyn SearchResultSerializer>
}

const RESULTS_COUNT: usize = 10;

impl InputHandler {

    pub fn new(prowlarr: ProwlarrClient,
               uuid_mapper: Box<dyn UuidMapper<TorrentMeta>>,
               downloads_tracker: Arc<DownloadsTracker>,
               allowed_users: Vec<u64>,
               sender: Box<dyn Sender>,
               search_result_serializer: Box<dyn SearchResultSerializer>) -> InputHandler {
        InputHandler {
            prowlarr,
            uuid_mapper,
            downloads_tracker,
            allowed_users,
            sender,
            search_result_serializer,
        }
    }

    pub async fn handle(&self, input: Box<dyn Input>) -> HandlingResult {
        let source = input.get_source();
        let destination = input.get_destination();
        let locale = input.get_locale();
        let reply_to_message = input.get_reply_to_message();
        if self.allowed_users.is_empty() || self.allowed_users.contains(&source) {
            self.sender.send_progress_indication(destination).await?;
            match input.get_command() {
                Command::Search(query) => self.search(source, destination, reply_to_message, &locale, &query).await?,
                Command::Download(uuid) => self.download(source, destination, &locale, &uuid).await?,
                Command::GetLink(uuid) => self.link(source, destination, &locale, &uuid).await?,
                Command::Help => self.sender.send_plain_message(destination, &t!("help", locale = &locale)).await?,
            }
        }
        Ok(())
    }

    async fn search(&self,
                    source: Source,
                    destination: Destination,
                    reply_to_message: ReplyToMessage,
                    locale: &Locale,
                    query: &SearchQuery
    ) -> HandlingResult {
        log::info!("from {} | Received search request \"{}\"", source, query);
        match self.prowlarr.search(query).await {
            Ok(results) => {
                let first_n_sorted_results: Vec<SearchResult> = sorted_by_seeders(results)
                    .into_iter()
                    .take(RESULTS_COUNT)
                    .collect();
                let bot_uuids = self.uuid_mapper.put_all(first_n_sorted_results
                    .iter()
                    .map(|a| a.into())
                    .collect()).await;
                match bot_uuids {
                    Ok(bot_uuids) => {
                        let response = first_n_sorted_results
                            .iter()
                            .enumerate()
                            .map(|(index, search_result)|
                                self.search_result_serializer.serialize(search_result, &bot_uuids[index], locale))
                            .reduce(|acc, e| acc + &e);
                        match response {
                            None => {
                                self.sender.send_plain_reply(destination, reply_to_message, &t!("no_results", locale = &locale, request = query)).await?;
                                log::info!("  to {} | Sent \"No results\" response", destination);
                            }
                            Some(response) => {
                                self.sender.send_reply(destination, reply_to_message, &response).await?;
                                log::info!("  to {} | Sent search response \"{}\"", destination, to_digest(&response));
                            }
                        }
                    }
                    Err(err) => self.handle_mapper_error(destination, locale, err).await?,
                }
            }
            Err(err) => self.handle_prowlarr_error(destination, locale, err).await?,
        }
        Ok(())
    }

    async fn handle_prowlarr_error(&self,
                                   destination: Destination,
                                   locale: &Locale,
                                   err: impl Display) -> HandlingResult {
        log::error!("  to {} | Error when interacting with Prowlarr: {}", destination, err);
        self.sender.send_plain_message(destination, &t!("prowlarr_error", locale = &locale)).await
    }

    async fn handle_mapper_error(&self,
                                 destination: Destination,
                                 locale: &Locale,
                                 err: MapperError) -> HandlingResult {
        log::error!("  to {} | {}", destination, err);
        self.sender.send_plain_message(destination, &t!("mapper_error", locale = locale)).await
    }

    async fn download(&self, source: Source, destination: Destination, locale: &Locale, uuid: &ItemUuid) -> HandlingResult {
        log::info!("from {} | Received download request for {}", source, uuid);
        match self.uuid_mapper.get(uuid).await {
            Ok(torrent_data) => match torrent_data {
                None => self.link_not_found(destination, locale, uuid).await?,
                Some(meta) => {
                    match self.prowlarr.download(&meta.indexer_id, &meta.guid).await {
                        Ok(response) => {
                            if response.status().is_success() {
                                self.sender.send_plain_message(destination, &t!("sent_to_download", locale = &locale)).await?;
                                log::info!("  to {} | Sent {} for downloading", destination, meta);
                                match meta.get_torrent_hash(&self.prowlarr).await {
                                    Ok(hash) => self.downloads_tracker.add(hash, destination, locale.clone()),
                                    Err(err) => {
                                        log::error!("  to {} | {}", destination, err);
                                    }
                                };
                            } else {
                                log::error!("  to {} | Download response from Prowlarr wasn't successful: {} {}",
                                    destination, response.status(), response.text().await.unwrap_or_default());
                                self.sender.send_plain_message(destination, &t!("could_not_send_to_download", locale = &locale)).await?;
                            }
                        }
                        Err(err) => self.handle_prowlarr_error(destination, locale, err).await?
                    }
                }
            }
            Err(err) => self.handle_mapper_error(destination, locale, err).await?,
        }
        Ok(())
    }

    async fn link(&self, source: Source, destination: Destination, locale: &Locale, uuid: &ItemUuid) -> HandlingResult {
        log::info!("from {} | Received get link request for {}", source, uuid);
        match self.uuid_mapper.get(uuid).await {
            Ok(torrent_data) => match torrent_data {
                None => self.link_not_found(destination, locale, uuid).await?,
                Some(torrent_data) => {
                    if torrent_data.magnet_url.is_some() { // todo this code resembles TorrentMeta.get_torrent_hash()
                        self.sender.send_magnet(destination, torrent_data.magnet_url.as_ref().unwrap()).await?;
                        log::info!("  to {} | Sent magnet link for {} ", destination, &torrent_data);
                    } else if torrent_data.download_url.is_some() {
                        match self.prowlarr.get_download_meta(torrent_data.download_url.as_ref().unwrap()).await {
                            Ok(content) => {
                                match content {
                                    DownloadMeta::MagnetLink(link) => {
                                        self.sender.send_magnet(destination, &link).await?;
                                        log::info!("  to {} | Sent magnet link for {} ", destination, &torrent_data);
                                    }
                                    DownloadMeta::TorrentFile(file) => {
                                        self.sender.send_torrent_file(destination, &format!("{}.torrent", uuid), file).await?;
                                        log::info!("  to {} | Sent .torrent file for {} ", destination, &torrent_data);
                                    }
                                }
                            }
                            Err(err) => self.handle_prowlarr_error(destination, locale, err).await?,
                        }
                    } else {
                        log::warn!("  to {} | Neither magnet nor download link exist for torrent {}", destination, torrent_data);
                        self.sender.send_plain_message(destination, &t!("link_not_found", locale = &locale)).await?;
                    }
                }
            }
            Err(err) => self.handle_mapper_error(destination, locale, err).await?,
        }
        Ok(())
    }

    async fn link_not_found(&self, destination: Destination, locale: &Locale, uuid: &ItemUuid) -> HandlingResult {
        log::warn!("  to {} | Link for uuid {} not found", destination, &uuid);
        self.sender.send_plain_message(destination, &t!("link_not_found", locale = locale)).await?;
        Ok(())
    }
}

fn sorted_by_seeders(mut results: Vec<SearchResult>) -> Vec<SearchResult> {
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
