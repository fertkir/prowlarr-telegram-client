use thiserror::Error;

pub mod input_handler;
pub mod traits;
pub mod downloads_tracker;
pub mod prowlarr;
pub mod util;
pub mod completion;
pub mod torrent_meta;
pub mod download_meta;

#[derive(Error, Debug)]
pub enum HandlingError {
    #[error("Error when sending a message: {}", .0)]
    SendError(String)
}

pub type HandlingResult = Result<(), HandlingError>;
