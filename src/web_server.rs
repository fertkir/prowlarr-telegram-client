use std::net::SocketAddr;
use std::sync::Arc;

use derive_more::Display;
use serde::Deserialize;
use tokio::task;
use warp::Filter;
use warp::reply::WithStatus;
use crate::core::downloads_tracker::DownloadsTracker;

use crate::core::ext::sender::Sender;
use crate::core::util;

#[derive(Deserialize, Display)]
#[display(fmt = "{{ hash: {}, name: {} }}", hash, name)]
struct CompletionRequest {
    hash: String,
    name: String
}

pub async fn run(sender: Arc<dyn Sender>, downloads_tracker: Arc<DownloadsTracker>) {
    if let Ok(port) = std::env::var("COMPLETE_PORT") {
        let filter = warp::put()
            .and(warp::path("complete"))
            .and(warp::body::json())
            .and(warp::any().map(move || downloads_tracker.clone()))
            .and(warp::any().map(move || sender.clone()))
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
                    sender: Arc<dyn Sender>) -> WithStatus<String> {
    log::info!("Received download completion notification for {}", request);
    for user in downloads_tracker.remove(request.hash).iter() {
        let sender = sender.clone();
        let download_name = request.name.clone();
        let chat_id = user.destination;
        let locale = user.locale.clone();
        task::spawn(async move {
            match sender.send_message(chat_id, &t!("download_complete", locale = &locale, name = download_name)).await {
                Ok(_) => {
                    log::info!("userId {} | Sent download complete notification for \"{}\"", chat_id, download_name);
                }
                Err(err) => {
                    log::error!("userId {} | Could not send download complete notification for \"{}\": {:?}",
                        chat_id, download_name, err);
                }
            };
        });
    }
    warp::reply::with_status(String::new(), warp::http::StatusCode::ACCEPTED)
}
