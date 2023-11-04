use std::net::SocketAddr;
use std::sync::Arc;

use teloxide::{Bot, dptree};
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::Update;
use teloxide::update_listeners::webhooks;

use crate::downloads_tracker::DownloadsTracker;
use crate::prowlarr::ProwlarrClient;
use crate::telegram::message_handler;
use crate::torrent_data::TorrentDataStore;
use crate::util;

pub async fn run(bot: Bot, downloads_tracker: Arc<DownloadsTracker>) {
    log::info!("Starting torrents bot...");

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler::handle));

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![
            Arc::new(ProwlarrClient::from_env()),
            Arc::new(TorrentDataStore::new()),
            downloads_tracker,
            get_allowed_users()])
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

fn get_allowed_users() -> Vec<u64> {
    std::env::var("ALLOWED_USERS")
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
        use crate::telegram::dispatcher::get_allowed_users;

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
                assert_eq!(get_allowed_users().get(0), Some(1000).as_ref());
            });
        }

        #[test]
        fn multiple_users() {
            temp_env::with_var("ALLOWED_USERS", Some("1000,2000,3000"), || {
                assert_eq!(get_allowed_users().len(), 3);
                assert_eq!(get_allowed_users().get(0), Some(1000).as_ref());
                assert_eq!(get_allowed_users().get(1), Some(2000).as_ref());
                assert_eq!(get_allowed_users().get(2), Some(3000).as_ref());
            });
        }
    }
}