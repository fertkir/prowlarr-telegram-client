pub mod input_handler;
pub mod traits;
pub mod downloads_tracker;
pub mod prowlarr;
pub mod util;
pub mod completion;

#[derive(Debug)]
pub enum HandlingError {
    SendError(String)
}

pub type HandlingResult = Result<(), HandlingError>;
