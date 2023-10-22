#[macro_use]
extern crate rust_i18n;

use byte_unit::Byte;
use rust_i18n::t;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::markdown::bold;
use teloxide::utils::markdown::link;

use prowlarr::ProwlarrClient;

use crate::prowlarr::SearchResult;

mod prowlarr;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting torrents bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![ProwlarrClient::from_env()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn message_handler(prowlarr: ProwlarrClient, bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(m) = msg.text() {
        if !m.starts_with("/") {
            log::info!("Received message \"{}\" from user {}", m, msg.chat.id);
            let mut results = prowlarr.search(m).await?;
            results.sort_unstable_by(|a, b| b.seeders.cmp(&a.seeders));
            let response = results
                .iter()
                .take(10)
                .map(SearchResult::to_msg)
                .fold(String::new(), |acc, e| acc + &e);
            bot.parse_mode(ParseMode::Markdown) // todo is it a good idea to call parse_mode() every time?
                .send_message(msg.chat.id, response).await?;
        } else if m.starts_with("/d_") {} else if m.starts_with("/m_") {} else {
            bot.send_message(msg.chat.id, t!("Help Message")).await?;
        }
    }
    Ok(())
}

impl SearchResult {
    fn to_msg(&self) -> String {
        let grabs = match self.grabs {
            None => String::new(), // todo do not create a heap object
            Some(grabs) => format!("{} {}", t!("Downloaded"), grabs)
        };
        format!("{}\n{}\nS {} | L {} | {} | {} {} | {} {}\n{} {}\n{} {}\n\n", self.title,
                link(&self.info_url, &t!("Description")),
                self.seeders, self.leechers, grabs, &t!("Reg"), self.publish_date.date_naive(),
                &t!("Size"), Byte::from_bytes(self.size).get_appropriate_unit(false),
                bold(&t!("Download")), "", &t!("Get link"), "")
    }
}