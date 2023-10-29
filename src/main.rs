#[macro_use]
extern crate rust_i18n;

use std::sync::Arc;

use teloxide::Bot;

use crate::downloads_tracker::DownloadsTracker;

mod prowlarr;
mod uuid_mapper;
mod torrent_data;
mod web_server;
mod downloads_tracker;
mod telegram;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    env_logger::init();
    let bot = Bot::from_env(); // add TELOXIDE_PROXY env var to proxy to Telegram
    let downloads_tracker = Arc::new(DownloadsTracker::new());
    tokio::join!(
        telegram::dispatcher::run(bot.clone(), downloads_tracker.clone()),
        web_server::run(bot, downloads_tracker));
}
