use std::env::VarError;

use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("User \"{username}\" not found")]
    UserNotFound { username: String },
    #[error("User with this username and/or email already exists")]
    UserAlreadyExists,
    #[error("Invaid credentials")]
    InvalidCredentials,
    #[error("Post not found")]
    PostNotFound,
    #[error("Forbidden: trying to edit another user's post")]
    Forbidden,
    #[error("SQL error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Migrate error: {0}")]
    MigrateError(#[from] MigrateError),
    #[error("Dotenvy error: {0}")]
    DotenvyError(#[from] dotenvy::Error),
    #[error("Var error: {0}")]
    VarError(#[from] VarError),
    #[error("Unable to create date/time")]
    InvalidDatetime,
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Hash error: {0}")]
    HashError(String),
    #[error("Token is invalid or expired")]
    InvalidToken,
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::HashError(value.to_string())
    }
}
