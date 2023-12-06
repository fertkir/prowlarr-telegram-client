use teloxide::prelude::Message;

use crate::core::ext::input::{Command, Destination, Input, Locale, Source};
use crate::core::ext::input::Command::{Download, GetLink, Help, Ignore, Search};

pub struct TelegramInput(Message);

impl TelegramInput {
    pub fn from(msg: Message) -> TelegramInput {
        TelegramInput(msg)
    }
}

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
