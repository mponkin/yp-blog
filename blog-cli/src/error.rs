use blog_client::error::BlogClientError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("BlogClientError {0}")]
    ClientError(#[from] BlogClientError),
    #[error("Token not found. Run register or login command first and repeat request")]
    TokenNotFound,
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
