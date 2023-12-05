#[derive(Debug)]
pub enum HandlingError {
    SendError(String)
}

pub type HandlingResult = Result<(), HandlingError>;
