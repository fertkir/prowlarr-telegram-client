use std::net::SocketAddr;
use std::sync::Arc;

use teloxide::{Bot, dptree};
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::prelude::{LoggingErrorHandler, Message, Update};
use teloxide::update_listeners::webhooks;

use crate::core::HandlingResult;
use crate::core::input_handler::InputHandler;
use crate::core::traits::input::{Command, Destination, Input, Locale, Source};
use crate::core::traits::input::Command::{Download, GetLink, Help, Ignore, Search};
use crate::core::util;

struct TelegramInput(Message);

impl Input for TelegramInput {
    fn get_command(&self) -> Command {
        if let Some(msg_text) = self.0.text() {
            return if !msg_text.starts_with('/') {
                Search(msg_text.to_string())
            } else if let Some(item_uuid) = msg_text.strip_prefix("/d_") {
                Download(item_uuid.to_string())
            } else if let Some(item_uuid) = msg_text.strip_prefix("/m_") {
                GetLink(item_uuid.to_string())
            } else {
                Help
            }
        }
        Ignore
    }

    fn get_source(&self) -> Source {
        self.0.from().map(|from| from.id.0).unwrap_or(0)
    }

    fn get_destination(&self) -> Destination {
        self.0.chat.id.0
    }

    fn get_locale(&self) -> Locale {
        self.0.from()
            .and_then(|u| u.language_code.clone())
            .unwrap_or_else(|| String::from("en"))
    }
}

pub async fn run(bot: Bot, input_handler: InputHandler) {
    log::info!("Starting torrents bot...");

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle));

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![Arc::new(input_handler)])
        .enable_ctrlc_handler()
        .build();
    if let (Ok(port), Ok(url)) = (std::env::var("WEBHOOK_PORT"), std::env::var("WEBHOOK_URL")) {
        let addr = SocketAddr::new(util::parse_ip("WEBHOOK_IP"), port.parse().unwrap());
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

async fn handle(input_handler: Arc<InputHandler>, msg: Message) -> HandlingResult {
    input_handler.handle(Box::new(TelegramInput(msg))).await
}
