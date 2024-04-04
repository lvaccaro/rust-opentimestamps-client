// Copyright (C) 2024 The OpenTimestamps developers

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error")]
    NetworkError(reqwest::Error),
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
