#[macro_use]
extern crate rust_i18n;

use rust_i18n::t;
use teloxide::prelude::*;

mod prowlarr_client;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting torrents bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler));
    // .branch(Update::filter_inline_query().endpoint(inline_queries_handler))
    // .branch(Update::filter_callback_query().endpoint(callback_queries_handler));

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .enable_ctrlc_handler()
        .build()
        .dispatch().await;
}

async fn message_handler(bot: Bot, msg: Message) -> HandlerResult {
    if let Some(m) = msg.text() {
        if !m.starts_with("/") {
            bot.send_message(msg.chat.id, format!("Received: {}", m)).await?;
        } else if m.starts_with("/d_") {} else if m.starts_with("/m_") {} else {
            bot.send_message(msg.chat.id, t!("Help Message")).await?;
        }
    }
    Ok(())
}
