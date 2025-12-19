use std::sync::Arc;

use actix_web::{Error, HttpMessage, dev::ServiceRequest, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::{
    domain::user::AuthenticatedUser,
    infrastructure::jwt::{Claims, JwtService},
};

impl From<Claims> for AuthenticatedUser {
    fn from(value: Claims) -> Self {
        Self {
            user_id: value.user_id,
            username: value.username,
        }
    }
}

pub async fn jwt_validator(
    request: ServiceRequest,
    auth: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    match request
        .app_data::<web::Data<Arc<JwtService>>>()
        .map(|jwt_service| jwt_service.verify_token(auth.token()))
    {
        Some(Ok(claims)) => {
            let user = AuthenticatedUser::from(claims);
            request.extensions_mut().insert(user);

            Ok(request)
        }
        Some(Err(_)) => Err((
            actix_web::error::ErrorUnauthorized("Invalid or expired token"),
            request,
        )),
        None => Err((
            actix_web::error::ErrorInternalServerError("JwtService is not configured"),
            request,
        )),
    }
}
