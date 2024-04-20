use std::sync::Arc;

use derive_more::Display;
use serde::Deserialize;
use tokio::task;

use crate::core::downloads_tracker::DownloadsTracker;
use crate::core::traits::sender::Sender;

#[derive(Deserialize, Display)]
#[display(fmt = "{{ hash: {}, name: {} }}", hash, name)]
pub struct CompletionRequest {
    hash: String,
    name: String,
}

pub async fn notify(request: CompletionRequest,
                    downloads_tracker: Arc<DownloadsTracker>,
                    sender: Arc<dyn Sender>) {
    log::info!("Received download completion notification for {}", request);
    for user in downloads_tracker.remove(request.hash).iter() {
        let sender = sender.clone();
        let download_name = request.name.clone();
        let chat_id = user.destination;
        let locale = user.locale.clone();
        task::spawn(async move {
            match sender.send_plain_message(chat_id, &t!("download_complete", locale = &locale, name = download_name)).await {
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
}
