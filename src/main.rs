#[macro_use]
extern crate rust_i18n;

use std::env;
use std::sync::Arc;

use teloxide::Bot;

use crate::core::downloads_tracker::DownloadsTracker;
use crate::core::input_handler::InputHandler;
use crate::core::prowlarr::ProwlarrClient;
use crate::ext::search_result_serializer::telegram::TgSearchResultSerializer;
use crate::ext::sender::telegram::TelegramSender;
use crate::ext::uuid_mapper;
use crate::torrent::torrent_meta::TorrentMeta;

mod torrent;
mod core;
mod ext;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    let bot = Bot::from_env();
    let downloads_tracker = Arc::new(DownloadsTracker::new());

    let input_handler = InputHandler::new(
        ProwlarrClient::from_env(),
        uuid_mapper::create::<TorrentMeta>(),
        downloads_tracker.clone(),
        get_allowed_users(),
        Box::new(TelegramSender::from(bot.clone())),
        Box::new(TgSearchResultSerializer)
    );

    tokio::join!(
        ext::input_handler::telegram::run(bot.clone(), input_handler),
        ext::completion::web::run(Arc::new(TelegramSender::from(bot)), downloads_tracker));
}

fn get_allowed_users() -> Vec<u64> {
    env::var("ALLOWED_USERS")
        .unwrap_or_default()
        .split(',')
        .filter(|user| !user.is_empty())
        .map(|user| user.parse::<u64>()
            .unwrap_or_else(|_| panic!("ALLOWED_USERS list must be a comma-separated \
                string of integers. Value \"{user}\" is unexpected")))
        .collect()
}

#[cfg(test)]
mod tests {
    mod allowed_users {
        use crate::get_allowed_users;

        #[test]
        fn empty_list_if_var_is_not_set() {
            temp_env::with_var_unset("ALLOWED_USERS", || {
                assert_eq!(get_allowed_users().len(), 0)
            });
        }

        #[test]
        #[should_panic(expected = "ALLOWED_USERS list must be a comma-separated \
                string of integers. Value \"aaa\" is unexpected")]
        fn incorrect_allowed_users_value() {
            temp_env::with_var("ALLOWED_USERS", Some("aaa"), || {
                get_allowed_users();
            });
        }

        #[test]
        fn one_user() {
            temp_env::with_var("ALLOWED_USERS", Some("1000"), || {
                assert_eq!(get_allowed_users().len(), 1);
                assert_eq!(get_allowed_users().first(), Some(1000).as_ref());
            });
        }

        #[test]
        fn multiple_users() {
            temp_env::with_var("ALLOWED_USERS", Some("1000,2000,3000"), || {
                assert_eq!(get_allowed_users().len(), 3);
                assert_eq!(get_allowed_users().first(), Some(1000).as_ref());
                assert_eq!(get_allowed_users().get(1), Some(2000).as_ref());
                assert_eq!(get_allowed_users().get(2), Some(3000).as_ref());
            });
        }
    }
}
