#[macro_use]
extern crate rust_i18n;

use std::sync::Arc;

use byte_unit::Byte;
use rust_i18n::t;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::markdown::bold;
use teloxide::utils::markdown::link;

use prowlarr::ProwlarrClient;

use crate::prowlarr::{DownloadParams, SearchResult};
use crate::uuid_mapper::UuidMapper;

mod prowlarr;
mod uuid_mapper;

const RESULTS_COUNT: usize = 10;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting torrents bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            Arc::new(ProwlarrClient::from_env()),
            Arc::new(UuidMapper::<DownloadParams>::new())])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn message_handler(prowlarr: Arc<ProwlarrClient>,
                         uuid_mapper: Arc<UuidMapper<DownloadParams>>,
                         bot: Bot,
                         msg: Message) -> ResponseResult<()> {
    if let Some(m) = msg.text() {
        let locale = get_locale(&msg);
        if !m.starts_with("/") {
            log::info!("Received message \"{}\" from user {}", m, msg.chat.id);
            match prowlarr.search(m).await {
                Ok(mut results) => {
                    results.sort_unstable_by(|a, b| b.seeders.cmp(&a.seeders));
                    let response = results
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
                            bot.send_message(msg.chat.id, t!("no_results", locale = &locale, request = m)).await?;
                        }
                        Some(response) => {
                            bot.send_message(msg.chat.id, response)
                                .parse_mode(ParseMode::Markdown)
                                .disable_web_page_preview(true)
                                .await?;
                        }
                    }
                }
                Err(err) => {
                    log::error!("Error when searching in prowlarr: {}", err);
                    bot.send_message(msg.chat.id, t!("prowlarr_error", locale = &locale)).await?;
                }
            }
        } else if m.starts_with("/d_") {
            match uuid_mapper.get(&m[3..]) {
                None => {
                    bot.send_message(msg.chat.id, t!("link_not_found", locale = &locale)).await?;
                }
                Some(params) => {
                    match prowlarr.download(&params).await {
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
                            log::error!("Error when searching in prowlarr: {}", err);
                            bot.send_message(msg.chat.id, t!("prowlarr_error", locale = &locale)).await?;
                        }
                    }
                }
            }
        } else if m.starts_with("/m_") {
            // todo implement
        } else {
            bot.send_message(msg.chat.id, t!("help", locale = &locale)).await?;
        }
    }
    Ok(())
}

fn get_locale(msg: &Message) -> String {
    msg.from()
        .map(|u| u.language_code.clone())
        .flatten()
        .unwrap_or(String::from("en"))
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
