use async_trait::async_trait;
use bytes::Bytes;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{InputFile, ParseMode};

use crate::core::ext::error::{HandlingError, HandlingResult};
use crate::core::ext::input::Destination;
use crate::core::ext::sender::Sender;

pub struct TelegramSender {
    bot: Bot
}

#[async_trait]
impl Sender for TelegramSender {
    async fn send_message(&self, destination: &Destination, message: &str) -> HandlingResult {
        self.bot.send_message(destination, message)
            .parse_mode(ParseMode::MarkdownV2)
            .disable_web_page_preview(true)
            .await
            .map(|message| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }

    async fn send_plain_message(&self, destination: &Destination, message: &str) -> HandlingResult {
        self.bot.send_message(destination, message)
            .await
            .map(|message| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }

    async fn send_magnet(&self, destination: &Destination, link: &str) -> HandlingResult {
        self.bot.send_message(destination, format!("```\n{}\n```", link))
            .parse_mode(ParseMode::MarkdownV2)
            .await
            .map(|message| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }

    async fn send_torrent_file(&self, destination: &Destination, filename: &str, file: Bytes) -> HandlingResult {
        let file = InputFile::memory(file)
            .file_name(filename);
        self.bot.send_document(destination, file)
            .await
            .map(|message| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }
}
