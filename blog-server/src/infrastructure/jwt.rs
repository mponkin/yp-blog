use chrono::{DateTime, TimeDelta, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::domain::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: DateTime<Utc>,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        let secret_bytes = secret.as_bytes();
        JwtService {
            encoding_key: EncodingKey::from_secret(secret_bytes),
            decoding_key: DecodingKey::from_secret(secret_bytes),
        }
    }

    pub fn generate_token(&self, user_id: i64, username: String) -> Result<String, AppError> {
        const TOKEN_LIFETIME: TimeDelta = TimeDelta::days(1);
        let expiration_time = Utc::now()
            .checked_add_signed(TOKEN_LIFETIME)
            .ok_or(AppError::InvalidDatetime)?;

        trace!("Generating token for {username} ({user_id}) with lifetime {expiration_time}");

        let claims = Claims {
            user_id,
            username,
            exp: expiration_time,
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(AppError::from)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let validation = Validation::default();
        decode::<Claims>(&token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(AppError::from)
    }
}
