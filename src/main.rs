#[macro_use]
extern crate rust_i18n;

use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::update_listeners::webhooks;

use prowlarr::ProwlarrClient;

use crate::prowlarr::DownloadParams;
use crate::uuid_mapper::UuidMapper;

mod prowlarr;
mod uuid_mapper;
mod message_handling;
mod util;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting torrents bot...");

    let bot = Bot::from_env(); // add TELOXIDE_PROXY env var to proxy to Telegram

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handling::message_handler));

    let webhook_listener = if let (Ok(port), Ok(url)) = (std::env::var("WEBHOOK_PORT"), std::env::var("WEBHOOK_URL")) {
        let addr = ([127, 0, 0, 1], port.parse().unwrap()).into();
        Some(webhooks::axum(bot.clone(), webhooks::Options::new(addr, reqwest::Url::parse(&url).unwrap()))
            .await
            .unwrap())
    } else {
        None
    };

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            Arc::new(ProwlarrClient::from_env()),
            Arc::new(UuidMapper::<DownloadParams>::new()),
            get_allowed_users()])
        .enable_ctrlc_handler()
        .build();
    if webhook_listener.is_some() {
        dispatcher.dispatch_with_listener(
            webhook_listener.unwrap(),
            LoggingErrorHandler::with_custom_text("An error from the update listener"))
            .await
    } else {
        dispatcher.dispatch().await;
    }
}

fn get_allowed_users() -> Vec<u64> {
    match std::env::var("ALLOWED_USERS") {
        Ok(users) => users
            .split(",")
            .map(|user| user.parse::<u64>().unwrap())
            .collect(),
        Err(_) => Vec::new()
    }
}
