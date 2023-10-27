use std::fmt::Display;
use std::sync::Arc;

use byte_unit::Byte;
use teloxide::Bot;
use teloxide::payloads::{SendMessage, SendMessageSetters};
use teloxide::prelude::{Message, Requester, ResponseResult};
use teloxide::requests::JsonRequest;
use teloxide::types::{ChatId, InputFile, ParseMode};
use teloxide::utils::markdown::bold;
use teloxide::utils::markdown::link;

use crate::prowlarr::{DownloadUrlContent, ProwlarrClient, SearchResult};
use crate::torrent_data::{TorrentData, TorrentDataStore};

const RESULTS_COUNT: usize = 10;

pub async fn message_handler(prowlarr: Arc<ProwlarrClient>,
                             torrent_data_store: Arc<TorrentDataStore>,
                             allowed_users: Vec<u64>,
                             bot: Bot,
                             msg: Message) -> ResponseResult<()> {
    if allowed_users.is_empty() || allowed_users.contains(&get_user_id(&msg)) {
        if let Some(msg_text) = msg.text() {
            let locale = get_locale(&msg);
            if !msg_text.starts_with("/") {
                search(&prowlarr, &torrent_data_store, &bot, &msg, msg_text, &locale).await?;
            } else if msg_text.starts_with("/d_") {
                download(&prowlarr, &torrent_data_store, &bot, &msg, &msg_text, &locale).await?;
            } else if msg_text.starts_with("/m_") {
                get_link(&prowlarr, &torrent_data_store, &bot, &msg, &msg_text, &locale).await?;
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
        .map(|u| u.language_code.clone())
        .flatten()
        .unwrap_or(String::from("en"))
}

async fn search(prowlarr: &Arc<ProwlarrClient>,
                torrent_data_store: &Arc<TorrentDataStore>,
                bot: &Bot,
                msg: &Message,
                msg_text: &str,
                locale: &String) -> ResponseResult<()> {
    log::info!("userId {} | Received search request \"{}\"", msg.chat.id, msg_text);
    match prowlarr.search(msg_text).await {
        Ok(results) => {
            let response = sorted_by_seeders(results)
                .iter()
                .take(RESULTS_COUNT)
                .map(|search_result| {
                    let bot_uuid = &torrent_data_store.put(TorrentData {
                        indexer_id: search_result.indexer_id,
                        download_url: search_result.download_url.clone(),
                        guid: search_result.guid.clone(),
                        magnet_url: search_result.magnet_url.clone(),
                    });
                    create_response(&search_result, &bot_uuid, &locale)
                })
                .reduce(|acc, e| acc + &e);
            match response {
                None => {
                    bot.send_message(msg.chat.id, t!("no_results", locale = &locale, request = msg_text)).await?;
                    log::info!("userId {} | Sent \"No results\" response", msg.chat.id);
                }
                Some(response) => {
                    let response_digest = to_digest(&response);
                    bot.send_message(msg.chat.id, response)
                        .parse_mode(ParseMode::Markdown)
                        .disable_web_page_preview(true)
                        .await?;
                    log::info!("userId {} | Sent search response \"{}\"", msg.chat.id, response_digest);
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

fn create_response(search_result: &SearchResult, bot_uuid: &str, locale: &str) -> String {
    let downloads = search_result.grabs
        .map(|grabs| format!("{} {}", t!("downloaded", locale = &locale), grabs))
        .unwrap_or_else(String::new);
    format!("{}\n{}\nS {} | L {} | {} | {} {} | {} {}\n{}: /d\\_{}\n{}: /m\\_{}\n\n",
            search_result.title, link(&search_result.info_url, &t!("description", locale = &locale)),
            search_result.seeders, search_result.leechers, downloads, &t!("registered", locale = &locale),
            search_result.publish_date.date_naive(), &t!("size", locale = &locale),
            Byte::from_bytes(search_result.size).get_appropriate_unit(false),
            bold(&t!("download", locale = &locale)), bot_uuid,
            &t!("get_link", locale = &locale), bot_uuid)
}

fn to_digest(str: &str) -> String {
    str.char_indices()
        .map(|(i, _)| i)
        .nth(100)
        .map(|end|str[0..end].to_string())
        .unwrap_or(str.to_string())
}

async fn handle_prowlarr_error(bot: &Bot,
                               msg: &Message,
                               locale: &String,
                               err: impl Display) -> ResponseResult<Message> {
    log::error!("Error when interacting with Prowlarr: {}", err);
    bot.send_message(msg.chat.id, t!("prowlarr_error", locale = &locale)).await
}

async fn download(prowlarr: &Arc<ProwlarrClient>,
                  torrent_data_store: &Arc<TorrentDataStore>,
                  bot: &Bot,
                  msg: &Message,
                  msg_text: &str,
                  locale: &String) -> ResponseResult<()> {
    match torrent_data_store.get(&msg_text[3..]) {
        None => {
            bot.send_message(msg.chat.id, t!("link_not_found", locale = &locale)).await?;
        },
        Some(torrent_data) => {
            match prowlarr.download(&torrent_data.indexer_id, &torrent_data.guid).await {
                Ok(response) => {
                    if response.status().is_success() {
                        bot.send_message(msg.chat.id, t!("sent_to_download", locale = &locale)).await?;
                    } else {
                        log::error!("Download response from Prowlarr wasn't successful: {} {}",
                                    response.status(), response.text().await.unwrap_or_default());
                        bot.send_message(msg.chat.id, t!("could_not_send_to_download", locale = &locale)).await?;
                    }
                }
                Err(err) => {
                    handle_prowlarr_error(bot, msg, locale, err).await?;
                }
            }
        }
    }
    Ok(())
}

async fn get_link(prowlarr: &Arc<ProwlarrClient>,
                  torrent_data_store: &Arc<TorrentDataStore>,
                  bot: &Bot,
                  msg: &Message,
                  msg_text: &str,
                  locale: &String) -> ResponseResult<()> {
    let torrent_uuid = &msg_text[3..];
    match torrent_data_store.get(torrent_uuid) {
        None => {
            bot.send_message(msg.chat.id, t!("link_not_found", locale = &locale)).await?;
        },
        Some(torrent_data) => {
            if torrent_data.magnet_url.is_some() {
                send_magnet(bot, msg.chat.id, &torrent_data.magnet_url.unwrap()).await?;
            } else if torrent_data.download_url.is_some() {
                match prowlarr.get_download_url_content(&torrent_data.download_url.unwrap()).await {
                    Ok(content) => {
                        match content {
                            DownloadUrlContent::MagnetLink(link) => {
                                send_magnet(bot, msg.chat.id, &link).await?;
                            }
                            DownloadUrlContent::TorrentFile(torrent_file) => {
                                let file = InputFile::memory(torrent_file)
                                    .file_name(format!("{}.torrent", torrent_uuid));
                                bot.send_document(msg.chat.id, file).await?;
                            }
                        }
                    }
                    Err(err) => {
                        handle_prowlarr_error(bot, msg, locale, err).await?;
                    }
                }
            } else {
                log::warn!("Neither magnet nor download link exist for torrent {}", &torrent_data.guid);
                bot.send_message(msg.chat.id, t!("link_not_found", locale = &locale)).await?;
            }
        }
    }
    Ok(())
}

fn send_magnet(bot: &Bot, chat_id: ChatId, link: &str) -> JsonRequest<SendMessage> {
    bot.send_message(chat_id, format!("```\n{}\n```", link))
        .parse_mode(ParseMode::Markdown)
}
