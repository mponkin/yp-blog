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
    /// GRPC status
    #[error("GRPC status: {0}")]
    GrpcStatus(#[from] tonic::Status),
    /// GRPC expected field not set in response
    #[error("GRPC field not set: {0}")]
    GrpcFieldNotSet(String),
    /// Can't create timestamp from millis
    #[error("Unable to create Datetime from: {0}")]
    IncorrectTimestamp(i64),
    /// Can't create GRPC metadata from token
    #[error("Unable to create GRPC metadata from token: {0}")]
    InvalidMetadata(#[from] InvalidMetadataValue),
}
