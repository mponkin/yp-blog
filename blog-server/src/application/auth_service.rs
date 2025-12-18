use std::sync::Arc;

use crate::{
    data::user_repository::UserRepository,
    domain::{error::AppError, user::UserAndToken},
    infrastructure::jwt::JwtService,
};

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

pub struct AuthService {
    user_repo: UserRepository,
    jwt_service: Arc<JwtService>,
    argon2: Argon2<'static>,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, jwt_service: Arc<JwtService>) -> Self {
        Self {
            user_repo,
            jwt_service,
            argon2: Argon2::default(),
        }
    }

    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<UserAndToken, AppError> {
        let salt = SaltString::generate(&mut OsRng);

        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        let user = self
            .user_repo
            .save_user(&username, &email, &password_hash)
            .await?;

        let token = self
            .jwt_service
            .generate_token(user.id, user.username.clone())?;

        Ok(UserAndToken { user, token })
    }

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<UserAndToken, AppError> {
        let user = self
            .user_repo
            .get_by_username(&username)
            .await?
            .ok_or(AppError::UserNotFound { username })?;

        let parsed_hash = PasswordHash::new(&user.password_hash)?;

        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::InvalidCredentials)?;

        let token = self
            .jwt_service
            .generate_token(user.id, user.username.clone())?;

        Ok(UserAndToken { user, token })
    }
}
