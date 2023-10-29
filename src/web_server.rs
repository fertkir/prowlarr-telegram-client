use std::sync::Arc;

use serde::Deserialize;
use teloxide::Bot;
use teloxide::prelude::Requester;
use warp::Filter;
use warp::reply::WithStatus;

use crate::downloads_tracker::DownloadsTracker;

#[derive(Deserialize, Debug)]
struct CompletionRequest {
    hash: String,
    name: String
}

pub async fn run(bot: Bot, downloads_tracker: Arc<DownloadsTracker>) {
    if let Ok(port) = std::env::var("COMPLETE_PORT") {
        let filter = warp::post()
            .and(warp::path("complete"))
            .and(warp::body::json())
            .and(warp::any().map(move || downloads_tracker.clone()))
            .and(warp::any().map(move || bot.clone()))
            .then(completion);
        warp::serve(filter)
            .run(([127, 0, 0, 1], port.parse().unwrap()))
            .await;
    }
}

async fn completion(request: CompletionRequest,
                    downloads_tracker: Arc<DownloadsTracker>,
                    bot: Bot) -> WithStatus<String> {
    log::info!("{:?}", request); // todo use something normal instead of Debug trait
    for user in downloads_tracker.remove(request.hash).iter() {
        match bot.send_message(user.chat_id, t!("download_complete", locale = &user.locale, name = request.name)).await { // todo send movie name
            Ok(_) => {}
            Err(err) => {
                log::error!("error: {}", err) // todo change error message
            }
        };
    }
    warp::reply::with_status(String::new(), warp::http::StatusCode::ACCEPTED) // todo return result before processing
}
