use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
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
        let fut = warp::serve(filter)
            .bind(addr).await
            .graceful(wait_for_shutdown())
            .run();
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

#[cfg(unix)]
async fn wait_for_shutdown() {
    let ctrl_c = tokio::signal::ctrl_c();
    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create SIGTERM signal handler: {}", e);
            // On failure, just wait for Ctrl+C
            ctrl_c.await.unwrap();
            return;
        }
    };

    tokio::select! {
        // Wait for Ctrl+C
        _ = ctrl_c => {
            log::info!("Received SIGINT (Ctrl+C). Shutting down...");
        },
        // Wait for SIGTERM
        _ = sigterm.recv() => {
            log::info!("Received SIGTERM. Shutting down...");
        },
    }
}

#[cfg(not(unix))]
async fn wait_for_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");
    log::info!("Received Ctrl+C. Shutting down...");
}
