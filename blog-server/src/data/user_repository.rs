use std::sync::Arc;

use sqlx::PgPool;

use crate::domain::{error::AppError, user::User};

pub struct UserRepository {
    db_pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }

    pub async fn get_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        sqlx::query_as(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn save_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AppError> {
        let query = "
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at
        ";

        const DUPLICATE_CODE: &str = "23505";

        sqlx::query_as(query)
            .bind(username)
            .bind(email)
            .bind(password_hash)
            .fetch_one(&*self.db_pool)
            .await
            .map_err(|err| {
                if let Some(e) = err.as_database_error()
                    && e.code().is_some_and(|code| code == DUPLICATE_CODE)
                {
                    AppError::UserAlreadyExists
                } else {
                    AppError::from(err)
                }
            })
    }
}
