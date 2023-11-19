use teloxide::prelude::Message;

use crate::core::ext::input::{Command, Destination, Input, Locale, Source};
use crate::core::ext::input::Command::{Download, GetLink, Help, Ignore, Search};

pub struct TelegramInput(Message);

impl Input for TelegramInput {
    fn get_command(&self) -> Command {
        if let Some(msg_text) = self.0.text() {
            if !msg_text.starts_with('/') {
                Search
            } else if msg_text.starts_with("/d_") {
                Download
            } else if msg_text.starts_with("/m_") {
                GetLink
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
        self.from()
            .and_then(|u| u.language_code.clone())
            .unwrap_or_else(|| String::from("en"))
    }
}
