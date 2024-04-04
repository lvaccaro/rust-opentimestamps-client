#[derive(Debug, thiserror::Error)]
pub enum Error {
    // TODO remove into specific errors
    #[error("Generic error {0}")]
    Generic(String),
}
impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::Generic(message)
    }
}
