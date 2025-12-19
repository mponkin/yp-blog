//! Blog client library errors

use tonic::metadata::errors::InvalidMetadataValue;

/// Error variants
#[derive(Debug, thiserror::Error)]
pub enum BlogClientError {
    /// Error while parsing string to URL
    #[error("Unable to parse url: {0}")]
    InvalidUrl(#[from] url::ParseError),
    /// Wrapper for errors from reqwest crate
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Happens if client tries to make request with token while token is not set
    #[error("Token is not set")]
    TokenNotSet,
    /// GRPC transport error
    #[error("GRPC transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),
    /// GRPC expected field not set in response
    #[error("GRPC field not set: {0}")]
    GrpcFieldNotSet(String),
    /// Can't create timestamp from millis
    #[error("Unable to create Datetime from: {0}")]
    IncorrectTimestamp(i64),
    /// Can't create GRPC metadata from token
    #[error("Unable to create GRPC metadata from token: {0}")]
    InvalidMetadata(#[from] InvalidMetadataValue),
    /// User with provided username or email already exists
    #[error("User with provided username or email already exists")]
    UserAlreadyExists,
    /// HTTP server returned unexpected code
    #[error("Unexpected HTTP response code {code}: {message}")]
    UnexpectedHttpResponse {
        /// Status code
        code: u16,
        /// Error message
        message: String,
    },
    /// gRPC server returned unexpected code
    #[error("Unexpected gRPC response {status_code}: {message}")]
    UnexpectedGrpcResponse {
        /// Status code
        status_code: u16,
        /// Error message
        message: String,
    },
    /// Login or password is incorrect
    #[error("Login or password is incorrect")]
    InvalidCredentials,
    /// Invalid token
    #[error("Invalid token")]
    InvalidToken,
    /// Forbidden
    #[error("Forbidden: trying to edit or delete post that does not belong to authorized user")]
    Forbidden,
    /// Not found
    #[error("Resource not found")]
    NotFound,
}

impl From<tonic::Status> for BlogClientError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::AlreadyExists => BlogClientError::UserAlreadyExists,
            tonic::Code::NotFound => BlogClientError::NotFound,
            tonic::Code::Unauthenticated => BlogClientError::InvalidToken,
            tonic::Code::PermissionDenied => BlogClientError::Forbidden,
            other => BlogClientError::UnexpectedGrpcResponse {
                status_code: other as u16,
                message: status.message().to_string(),
            },
        }
    }
}
