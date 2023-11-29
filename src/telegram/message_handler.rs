use std::fmt::Display;
use std::sync::Arc;

use teloxide::Bot;
use teloxide::payloads::{SendMessage, SendMessageSetters};
use teloxide::prelude::{Message, Requester, ResponseResult};
use teloxide::requests::JsonRequest;
use teloxide::types::{ChatId, InputFile, ParseMode};

use crate::downloads_tracker::DownloadsTracker;
use crate::prowlarr::{ProwlarrClient, SearchResult};
use crate::torrent::download_meta::{DownloadMeta, DownloadMetaProvider};
use crate::torrent::torrent_meta::TorrentMeta;
use crate::uuid_mapper::{MapperError, UuidMapper};

const RESULTS_COUNT: usize = 10;

pub async fn handle(prowlarr: Arc<ProwlarrClient>,
                    torrent_data_store: Arc<dyn UuidMapper<TorrentMeta>>,
                    downloads_tracker: Arc<DownloadsTracker>,
                    allowed_users: Vec<u64>,
                    bot: Bot,
                    msg: Message) -> ResponseResult<()> {
    if allowed_users.is_empty() || allowed_users.contains(&get_user_id(&msg)) {
        if let Some(msg_text) = msg.text() {
            let locale = get_locale(&msg);
            if !msg_text.starts_with('/') {
                search(&prowlarr, torrent_data_store, &bot, &msg, msg_text, &locale).await?;
            } else if msg_text.starts_with("/d_") {
                download(&prowlarr, torrent_data_store, &downloads_tracker, &bot, &msg, msg_text, &locale).await?;
            } else if msg_text.starts_with("/m_") {
                get_link(prowlarr.as_ref(), torrent_data_store, &bot, &msg, msg_text, &locale).await?;
            } else {
                bot.send_message(msg.chat.id, t!("help", locale = &locale)).await?;
            }
        }
    }
    Ok(())
}

fn get_user_id(msg: &Message) -> u64 {
    msg.from().map(|from| from.id.0).unwrap_or(0)
}

fn get_locale(msg: &Message) -> String {
    msg.from()
        .and_then(|u| u.language_code.clone())
        .unwrap_or_else(|| String::from("en"))
}

async fn search(prowlarr: &ProwlarrClient,
                torrent_data_store: Arc<dyn UuidMapper<TorrentMeta>>,
                bot: &Bot,
                msg: &Message,
                msg_text: &str,
                locale: &String) -> ResponseResult<()> {
    log::info!("userId {} | Received search request \"{}\"", msg.chat.id, msg_text);
    match prowlarr.search(msg_text).await {
        Ok(results) => {
            let first_n_sorted_results: Vec<SearchResult> = sorted_by_seeders(results)
                .into_iter()
                .take(RESULTS_COUNT)
                .collect();
            let bot_uuids = torrent_data_store.put_all(first_n_sorted_results
                .iter()
                .map(|a| a.into())
                .collect()).await;
            match bot_uuids {
                Ok(bot_uuids) => {
                    let response = first_n_sorted_results
                        .iter()
                        .enumerate()
                        .map(|(index, search_result)|
                            search_result.to_message(&bot_uuids[index], locale))
                        .reduce(|acc, e| acc + &e);
                    match response {
                        None => {
                            bot.send_message(msg.chat.id, t!("no_results", locale = &locale, request = msg_text)).await?;
                            log::info!("userId {} | Sent \"No results\" response", msg.chat.id);
                        }
                        Some(response) => {
                            let response_digest = to_digest(&response);
                            bot.send_message(msg.chat.id, response)
                                .parse_mode(ParseMode::MarkdownV2)
                                .disable_web_page_preview(true)
                                .await?;
                            log::info!("userId {} | Sent search response \"{}\"", msg.chat.id, response_digest);
                        }
                    }
                }
                Err(err) => {
                    handle_mapper_error(bot, msg, locale, err).await?;
                }
            }
        }
        Err(err) => {
            handle_prowlarr_error(bot, msg, locale, err).await?;
        }
    }
    Ok(())
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

async fn handle_prowlarr_error(bot: &Bot,
                               msg: &Message,
                               locale: &String,
                               err: impl Display) -> ResponseResult<Message> {
    log::error!("userId {} | Error when interacting with Prowlarr: {}", msg.chat.id, err);
    bot.send_message(msg.chat.id, t!("prowlarr_error", locale = &locale)).await
}

async fn handle_mapper_error(bot: &Bot,
                             msg: &Message,
                             locale: &String,
                             err: MapperError) -> ResponseResult<Message> {
    log::error!("userId {} | Error when interacting with mapper: {:?}", msg.chat.id, err);
    bot.send_message(msg.chat.id, t!("mapper_error", locale = &locale)).await
}

async fn download(prowlarr: &ProwlarrClient,
                  torrent_data_store: Arc<dyn UuidMapper<TorrentMeta>>,
                  downloads_tracker: &DownloadsTracker,
                  bot: &Bot,
                  msg: &Message,
                  msg_text: &str,
                  locale: &String) -> ResponseResult<()> {
    log::info!("userId {} | Received download request for {}", msg.chat.id, msg_text);
    match torrent_data_store.get(&msg_text[3..]).await {
        Ok(torrent_data) => match torrent_data {
            None => {
                log::warn!("userId {} | Link {} expired", msg.chat.id, msg_text);
                bot.send_message(msg.chat.id, t!("link_not_found", locale = &locale)).await?;
            }
            Some(torrent_data) => {
                match prowlarr.download(&torrent_data.indexer_id, &torrent_data.guid).await {
                    Ok(response) => {
                        if response.status().is_success() {
                            bot.send_message(msg.chat.id, t!("sent_to_download", locale = &locale)).await?;
                            log::info!("userId {} | Sent {} for downloading", msg.chat.id, torrent_data);
                            match torrent_data.get_torrent_hash(prowlarr).await {
                                Ok(hash) => {
                                    downloads_tracker.add(hash, msg.chat.id, locale.clone());
                                }
                                Err(err) => {
                                    log::error!("userId {} | {}", msg.chat.id, err);
                                }
                            };
                        } else {
                            log::error!("userId {} | Download response from Prowlarr wasn't successful: {} {}",
                            msg.chat.id, response.status(), response.text().await.unwrap_or_default());
                            bot.send_message(msg.chat.id, t!("could_not_send_to_download", locale = &locale)).await?;
                        }
                    }
                    Err(err) => {
                        handle_prowlarr_error(bot, msg, locale, err).await?;
                    }
                }
            }
        }
        Err(err) => {
            handle_mapper_error(bot, msg, locale, err).await?;
        }
    }
    Ok(())
}

async fn get_link(download_meta_provider: &impl DownloadMetaProvider,
                  torrent_data_store: Arc<dyn UuidMapper<TorrentMeta>>,
                  bot: &Bot,
                  msg: &Message,
                  msg_text: &str,
                  locale: &String) -> ResponseResult<()> {
    log::info!("userId {} | Received get link request for {}", msg.chat.id, msg_text);
    let torrent_uuid = &msg_text[3..];
    match torrent_data_store.get(torrent_uuid).await {
        Ok(torrent_data) => match torrent_data {
            None => {
                log::warn!("userId {} | Link {} expired", msg.chat.id, msg_text);
                bot.send_message(msg.chat.id, t!("link_not_found", locale = &locale)).await?;
            }
            Some(torrent_data) => {
                if torrent_data.magnet_url.is_some() {
                    send_magnet(bot, msg.chat.id, torrent_data.magnet_url.as_ref().unwrap()).await?;
                    log::info!("userId {} | Sent magnet link for {} ", msg.chat.id, &torrent_data);
                } else if torrent_data.download_url.is_some() {
                    match download_meta_provider.get_download_meta(torrent_data.download_url.as_ref().unwrap()).await {
                        Ok(content) => {
                            match content {
                                DownloadMeta::MagnetLink(link) => {
                                    send_magnet(bot, msg.chat.id, &link).await?;
                                    log::info!("userId {} | Sent magnet link for {} ", msg.chat.id, &torrent_data);
                                }
                                DownloadMeta::TorrentFile(torrent_file) => {
                                    let file = InputFile::memory(torrent_file)
                                        .file_name(format!("{}.torrent", torrent_uuid));
                                    bot.send_document(msg.chat.id, file).await?;
                                    log::info!("userId {} | Sent .torrent file for {} ", msg.chat.id, &torrent_data);
                                }
                            }
                        }
                        Err(err) => {
                            handle_prowlarr_error(bot, msg, locale, err).await?;
                        }
                    }
                } else {
                    log::warn!("userId {} | Neither magnet nor download link exist for torrent {}", msg.chat.id, torrent_data);
                    bot.send_message(msg.chat.id, t!("link_not_found", locale = &locale)).await?;
                }
            }
        }
        Err(err) => {
            handle_mapper_error(bot, msg, locale, err).await?;
        }
    }
    Ok(())
}

fn send_magnet(bot: &Bot, chat_id: ChatId, link: &str) -> JsonRequest<SendMessage> {
    bot.send_message(chat_id, format!("```\n{}\n```", link))
        .parse_mode(ParseMode::MarkdownV2)
}
