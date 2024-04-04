#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ots error")]
    InvalidOts(ots::error::Error),
    #[error("IO error")]
    IOError,
    #[error("Invalid file error")]
    InvalidFile,
    // TODO remove into specific errors
    #[error("Generic error {0}")]
    Generic(String),
}
impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::Generic(message)
    }
}
