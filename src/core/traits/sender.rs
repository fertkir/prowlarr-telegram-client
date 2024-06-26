use async_trait::async_trait;
use bytes::Bytes;

use crate::core::HandlingResult;
use crate::core::traits::input::{Destination, ReplyToMessage};

#[async_trait]
pub trait Sender: Send + Sync {
    async fn send_reply(&self, destination: Destination, reply_to_message: ReplyToMessage, message: &str) -> HandlingResult;
    async fn send_progress_indication(&self, destination: Destination) -> HandlingResult;
    async fn send_plain_message(&self, destination: Destination, message: &str) -> HandlingResult;
    async fn send_plain_reply(&self, destination: Destination, reply_to_message: ReplyToMessage, message: &str) -> HandlingResult;
    async fn send_magnet(&self, destination: Destination, link: &str) -> HandlingResult;
    async fn send_torrent_file(&self, destination: Destination, filename: &str, file: Bytes) -> HandlingResult;
}
