//! Blog client library errors

/// Error variants
#[derive(Debug, thiserror::Error)]
pub enum BlogClientError {
    /// Error while parsing string to URL
    #[error("Unable to pase url: {0}")]
    InvalidUrl(#[from] url::ParseError),
    /// Wrapper for errors from reqwest crate
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Happens if client tries to make request with token while token is not set
    #[error("Token is not set")]
    TokenNotSet,
}
