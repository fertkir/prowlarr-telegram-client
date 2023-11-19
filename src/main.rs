#[macro_use]
extern crate rust_i18n;

use std::env;
use std::sync::Arc;

use teloxide::Bot;

use crate::downloads_tracker::DownloadsTracker;

mod prowlarr;
mod web_server;
mod telegram;
mod util;
mod torrent;
mod core;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    let bot = Bot::from_env();
    let downloads_tracker = Arc::new(DownloadsTracker::new());
    tokio::join!(
        telegram::dispatcher::run(bot.clone(), downloads_tracker.clone()),
        web_server::run(bot, downloads_tracker));
}
