use std::net::SocketAddr;
use std::sync::Arc;

use warp::Filter;
use warp::reply::WithStatus;

use crate::core::{completion, util};
use crate::core::completion::CompletionRequest;
use crate::core::downloads_tracker::DownloadsTracker;
use crate::core::traits::sender::Sender;

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
    completion::notify(request, downloads_tracker, sender).await;
    warp::reply::with_status(String::new(), warp::http::StatusCode::ACCEPTED)
}
