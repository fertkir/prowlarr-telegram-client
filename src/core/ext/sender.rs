use async_trait::async_trait;
use bytes::Bytes;

use crate::core::ext::error::HandlingResult;
use crate::core::ext::input::{Destination, ItemUuid};

#[async_trait]
pub trait Sender {
    async fn send_message(&self, destination: &Destination, message: &str) -> HandlingResult;
    async fn send_plain_message(&self, destination: &Destination, message: &str) -> HandlingResult;
    async fn send_magnet(&self, destination: &Destination, link: &str) -> HandlingResult;
    async fn send_torrent_file(&self, destination: &Destination, filename: &str, file: Bytes) -> HandlingResult;
}
