use std::net::SocketAddr;
use std::sync::Arc;

use serde::Deserialize;
use teloxide::Bot;
use teloxide::prelude::Requester;
use tokio::task;
use warp::Filter;
use warp::reply::WithStatus;

use crate::downloads_tracker::DownloadsTracker;
use crate::util;

#[derive(Deserialize, Debug)]
struct CompletionRequest {
    hash: String,
    name: String
}

pub async fn run(bot: Bot, downloads_tracker: Arc<DownloadsTracker>) {
    if let Ok(port) = std::env::var("COMPLETE_PORT") {
        let filter = warp::put()
            .and(warp::path("complete"))
            .and(warp::body::json())
            .and(warp::any().map(move || downloads_tracker.clone()))
            .and(warp::any().map(move || bot.clone()))
            .then(completion);
        let addr = SocketAddr::new(util::parse_ip("COMPLETE_IP"), port.parse().unwrap());
        let (_, fut) = warp::serve(filter)
            .bind_with_graceful_shutdown(addr, async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to listen to shutdown signal");
            });
        log::info!("Server::run; addr={}", addr);
        log::info!("listening on http://{}", addr);
        fut.await;
    }
}

async fn completion(request: CompletionRequest,
                    downloads_tracker: Arc<DownloadsTracker>,
                    bot: Bot) -> WithStatus<String> {
    log::info!("{:?}", request); // todo use something normal instead of Debug trait
    for user in downloads_tracker.remove(request.hash).iter() {
        let bot = bot.clone();
        let download_name = request.name.clone();
        let chat_id = user.chat_id.clone();
        let locale = user.locale.clone();
        task::spawn(async move {
            match bot.send_message(chat_id, t!("download_complete", locale = &locale, name = download_name)).await {
                Ok(_) => {}
                Err(err) => {
                    log::error!("error: {}", err) // todo change error message
                }
            };
        });
    }
    warp::reply::with_status(String::new(), warp::http::StatusCode::ACCEPTED)
}
