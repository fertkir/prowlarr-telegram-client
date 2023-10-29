#[macro_use]
extern crate rust_i18n;

use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::update_listeners::webhooks;

use prowlarr::ProwlarrClient;

use crate::torrent_data::TorrentDataStore;

mod prowlarr;
mod uuid_mapper;
mod message_handling;
mod util;
mod torrent_data;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting torrents bot...");

    let bot = Bot::from_env(); // add TELOXIDE_PROXY env var to proxy to Telegram

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handling::message_handler));

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![
            Arc::new(ProwlarrClient::from_env()),
            Arc::new(TorrentDataStore::new()),
            get_allowed_users()])
        .enable_ctrlc_handler()
        .build();
    if let (Ok(port), Ok(url)) = (std::env::var("WEBHOOK_PORT"), std::env::var("WEBHOOK_URL")) {
        let addr = ([127, 0, 0, 1], port.parse().unwrap()).into();
        let webhook_listener = webhooks::axum(bot, webhooks::Options::new(addr, reqwest::Url::parse(&url).unwrap()))
            .await
            .unwrap();
        dispatcher.dispatch_with_listener(
            webhook_listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"))
            .await
    } else {
        dispatcher.dispatch().await;
    }
}

fn get_allowed_users() -> Vec<u64> {
    std::env::var("ALLOWED_USERS")
        .unwrap_or_default()
        .split(',')
        .filter(|user| !user.is_empty())
        .map(|user| user.parse::<u64>()
            .unwrap_or_else(|_| panic!("ALLOWED_USERS list must be a comma-separated \
                string of integers. Value \"{user}\" is unexpected")))
        .collect()
}
