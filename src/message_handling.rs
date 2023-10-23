use std::sync::Arc;

use byte_unit::Byte;
use reqwest::Error;
use teloxide::Bot;
use teloxide::payloads::{SendMessage, SendMessageSetters};
use teloxide::prelude::{Message, Requester, ResponseResult};
use teloxide::requests::JsonRequest;
use teloxide::types::ParseMode;
use teloxide::utils::markdown::bold;
use teloxide::utils::markdown::link;

use crate::prowlarr::{DownloadParams, ProwlarrClient, SearchResult};
use crate::uuid_mapper::UuidMapper;

const RESULTS_COUNT: usize = 10;

pub async fn message_handler(prowlarr: Arc<ProwlarrClient>,
                             uuid_mapper: Arc<UuidMapper<DownloadParams>>,
                             allowed_users: Vec<u64>,
                             bot: Bot,
                             msg: Message) -> ResponseResult<()> {
    if allowed_users.is_empty() || allowed_users.contains(&get_user_id(&msg)) {
        if let Some(msg_text) = msg.text() {
            let locale = get_locale(&msg);
            if !msg_text.starts_with("/") {
                search(&prowlarr, &uuid_mapper, &bot, &msg, msg_text, &locale).await?;
            } else if msg_text.starts_with("/d_") {
                download(prowlarr, uuid_mapper, &bot, &msg, &msg_text, &locale).await?;
            } else if msg_text.starts_with("/m_") {
                // todo implement
            } else {
                bot.send_message(msg.chat.id, t!("help", locale = &locale)).await?;
            }
        }
    }
    Ok(())
}

fn get_user_id(msg: &Message) -> u64 {
    msg.from().map(|from|from.id.0).unwrap_or(0)
}

fn get_locale(msg: &Message) -> String {
    msg.from()
        .map(|u| u.language_code.clone())
        .flatten()
        .unwrap_or(String::from("en"))
}

async fn search(prowlarr: &Arc<ProwlarrClient>,
                uuid_mapper: &Arc<UuidMapper<DownloadParams>>,
                bot: &Bot,
                msg: &Message,
                msg_text: &str,
                locale: &String) -> ResponseResult<Message> {
    log::info!("Received message \"{}\" from user {}", msg_text, msg.chat.id);
    match prowlarr.search(msg_text).await {
        Ok(results) => {
            let response = sorted_by_seeders(results)
                .iter()
                .take(RESULTS_COUNT)
                .map(|search_result| {
                    let bot_uuid = &uuid_mapper.put(DownloadParams {
                        indexer_id: search_result.indexer_id,
                        guid: search_result.guid.clone(),
                    });
                    create_response(&search_result, &bot_uuid, &locale)
                })
                .reduce(|acc, e| acc + &e);
            match response {
                None => {
                    bot.send_message(msg.chat.id, t!("no_results", locale = &locale, request = msg_text))
                }
                Some(response) => {
                    bot.send_message(msg.chat.id, response)
                        .parse_mode(ParseMode::Markdown)
                        .disable_web_page_preview(true)
                }
            }
        }
        Err(err) => handle_prowlarr_error(bot, msg, locale, err)
    }.await
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

fn handle_prowlarr_error(bot: &Bot, msg: &Message, locale: &String, err: Error) -> JsonRequest<SendMessage> {
    log::error!("Error when searching in Prowlarr: {}", err);
    bot.send_message(msg.chat.id, t!("prowlarr_error", locale = &locale))
}

async fn download(prowlarr: Arc<ProwlarrClient>,
                  uuid_mapper: Arc<UuidMapper<DownloadParams>>,
                  bot: &Bot,
                  msg: &Message,
                  msg_text: &str,
                  locale: &String) -> ResponseResult<Message> {
    match uuid_mapper.get(&msg_text[3..]) {
        None => {
            bot.send_message(msg.chat.id, t!("link_not_found", locale = &locale))
        }
        Some(params) => {
            match prowlarr.download(&params).await {
                Ok(response) => {
                    if response.status().is_success() {
                        bot.send_message(msg.chat.id, t!("sent_to_download", locale = &locale))
                    } else {
                        log::error!("Download response from Prowlarr wasn't successful: {} {}",
                                    response.status(), response.text().await.unwrap_or_default());
                        bot.send_message(msg.chat.id, t!("could_not_send_to_download", locale = &locale))
                    }
                }
                Err(err) => handle_prowlarr_error(bot, msg, locale, err)
            }
        }
    }.await
}
