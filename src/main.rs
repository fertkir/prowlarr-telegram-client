#[macro_use]
extern crate rust_i18n;

use std::sync::Arc;

use teloxide::prelude::*;

use prowlarr::ProwlarrClient;

use crate::prowlarr::DownloadParams;
use crate::uuid_mapper::UuidMapper;

mod prowlarr;
mod uuid_mapper;
mod message_handling;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting torrents bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handling::message_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            Arc::new(ProwlarrClient::from_env()),
            Arc::new(UuidMapper::<DownloadParams>::new())])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
