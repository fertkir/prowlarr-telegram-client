use std::sync::Arc;

use teloxide::Bot;
use teloxide::prelude::Requester;
use warp::Filter;
use warp::reply::WithStatus;

use crate::downloads_tracker::DownloadsTracker;

pub async fn run(bot: Bot, downloads_tracker: Arc<DownloadsTracker>) {
    if let Ok(port) = std::env::var("COMPLETE_PORT") {
        let filter = warp::post()
            .and(warp::path("complete"))
            .and(warp::path::param::<String>())
            .and(warp::any().map(move || downloads_tracker.clone()))
            .and(warp::any().map(move || bot.clone()))
            .then(completion);
        warp::serve(filter)
            .run(([127, 0, 0, 1], port.parse().unwrap()))
            .await;
    }
}

async fn completion(torrent_hash: String,
                    downloads_tracker: Arc<DownloadsTracker>,
                    bot: Bot) -> WithStatus<String> {
    log::info!("hash {}", torrent_hash); // todo change log
    for chat_id in downloads_tracker.remove(torrent_hash).iter() {
        match bot.send_message(*chat_id, "hash").await { // todo send movie name
            Ok(_) => {}
            Err(err) => {
                log::error!("error: {}", err) // todo change error message
            }
        };
    }
    warp::reply::with_status(String::new(), warp::http::StatusCode::ACCEPTED) // todo return result before processing
}
