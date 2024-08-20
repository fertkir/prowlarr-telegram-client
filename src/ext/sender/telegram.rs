use async_trait::async_trait;
use bytes::Bytes;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ChatAction, ChatId, InputFile, LinkPreviewOptions, MessageId, ParseMode, ReplyParameters};
use teloxide::Bot;

use crate::core::traits::input::{Destination, ReplyToMessage};
use crate::core::traits::sender::Sender;
use crate::core::HandlingError;
use crate::core::HandlingResult;

#[derive(Clone)]
pub struct TelegramSender {
    bot: Bot
}

impl TelegramSender {
    pub fn from(bot: Bot) -> TelegramSender {
        TelegramSender { bot }
    }
}

#[async_trait]
impl Sender for TelegramSender {
    async fn send_reply(&self, destination: Destination, reply_to_message: ReplyToMessage, message: &str) -> HandlingResult {
        self.bot.send_message(ChatId(destination), message)
            .reply_parameters(ReplyParameters::new(MessageId(reply_to_message)))
            .parse_mode(ParseMode::MarkdownV2)
            .link_preview_options(LinkPreviewOptions {
                is_disabled: true,
                url: None,
                prefer_small_media: false,
                prefer_large_media: false,
                show_above_text: false,
            })
            .await
            .map(|_| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }

    async fn send_progress_indication(&self, destination: Destination) -> HandlingResult {
        self.bot.send_chat_action(ChatId(destination), ChatAction::Typing)
            .await
            .map(|_| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }

    async fn send_plain_message(&self, destination: Destination, message: &str) -> HandlingResult {
        self.bot.send_message(ChatId(destination), message)
            .await
            .map(|_| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }

    async fn send_plain_reply(&self, destination: Destination, reply_to_message: ReplyToMessage, message: &str) -> HandlingResult {
        self.bot.send_message(ChatId(destination), message)
            .reply_parameters(ReplyParameters::new(MessageId(reply_to_message)))
            .await
            .map(|_| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }

    async fn send_magnet(&self, destination: Destination, link: &str) -> HandlingResult {
        self.bot.send_message(ChatId(destination), format!("```\n{}\n```", link))
            .parse_mode(ParseMode::MarkdownV2)
            .await
            .map(|_| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }

    async fn send_torrent_file(&self, destination: Destination, filename: &str, file: Bytes) -> HandlingResult {
        let file = InputFile::memory(file)
            .file_name(filename.to_string());
        self.bot.send_document(ChatId(destination), file)
            .await
            .map(|_| {})
            .map_err(|err| HandlingError::SendError(err.to_string()))
    }
}
