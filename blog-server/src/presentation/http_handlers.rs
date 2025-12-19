use std::sync::Arc;

use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, HttpResponseBuilder, ResponseError,
    http::StatusCode,
    web::{self, Data},
};
use serde::Serialize;

use crate::{
    application::{auth_service::AuthService, blog_service::BlogService},
    domain::{
        error::AppError,
        post::{CreatePostParams, GetPostsParams, GetPostsResponse, UpdatePostParams},
        user::{AuthenticatedUser, CreateUserParams, LoginParams},
    },
};

pub async fn register(
    auth_service: Data<Arc<AuthService>>,
    request: web::Json<CreateUserParams>,
) -> Result<HttpResponse, AppError> {
    let user_and_token = auth_service
        .register(request.0.username, request.0.email, request.0.password)
        .await?;

    Ok(HttpResponseBuilder::new(StatusCode::CREATED).json(user_and_token))
}

pub async fn login(
    auth_service: Data<Arc<AuthService>>,
    request: web::Json<LoginParams>,
) -> Result<HttpResponse, AppError> {
    let user_and_token = auth_service
        .login(request.0.username, request.0.password)
        .await?;

    Ok(HttpResponseBuilder::new(StatusCode::OK).json(user_and_token))
}

pub async fn create_post(
    req: HttpRequest,
    blog_service: Data<Arc<BlogService>>,
    post_data: web::Json<CreatePostParams>,
) -> Result<HttpResponse, AppError> {
    let user_id = try_get_user_id(req)?;
    let params: CreatePostParams = post_data.into_inner();

    let post = blog_service
        .create_post(params.title, params.content, user_id)
        .await?;

    Ok(HttpResponseBuilder::new(StatusCode::CREATED).json(post))
}

pub async fn get_post(
    path: web::Path<i64>,
    blog_service: Data<Arc<BlogService>>,
) -> Result<HttpResponse, AppError> {
    let post_id = path.into_inner();

    let post = blog_service.get_post(post_id).await?;

    Ok(HttpResponseBuilder::new(StatusCode::OK).json(post))
}

pub async fn update_post(
    req: HttpRequest,
    path: web::Path<i64>,
    blog_service: Data<Arc<BlogService>>,
    post_data: web::Json<UpdatePostParams>,
) -> Result<HttpResponse, AppError> {
    let user_id = try_get_user_id(req)?;
    let post_id = path.into_inner();
    let post_data = post_data.into_inner();

    let post = blog_service
        .update_post(post_id, post_data.title, post_data.content, user_id)
        .await?;

    Ok(HttpResponseBuilder::new(StatusCode::OK).json(post))
}

pub async fn delete_post(
    req: HttpRequest,
    path: web::Path<i64>,
    blog_service: Data<Arc<BlogService>>,
) -> Result<HttpResponse, AppError> {
    let user_id = try_get_user_id(req)?;
    let post_id = path.into_inner();

    blog_service.delete_post(post_id, user_id).await?;

    Ok(HttpResponseBuilder::new(StatusCode::NO_CONTENT).finish())
}

pub async fn get_posts(
    params: web::Query<GetPostsParams>,
    blog_service: Data<Arc<BlogService>>,
) -> Result<HttpResponse, AppError> {
    let (posts, total_posts_count) = blog_service.get_posts(params.limit, params.offset).await?;

    let response = GetPostsResponse {
        posts,
        total_posts: total_posts_count,
        limit: params.limit,
        offset: params.offset,
    };

    Ok(HttpResponseBuilder::new(StatusCode::OK).json(response))
}

fn try_get_user_id(req: HttpRequest) -> Result<i64, AppError> {
    match req.extensions().get::<AuthenticatedUser>() {
        Some(user) => Ok(user.user_id),
        None => Err(AppError::InvalidToken),
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = match self {
            AppError::UserNotFound { .. } => StatusCode::UNAUTHORIZED,
            AppError::UserAlreadyExists => StatusCode::CONFLICT,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::PostNotFound => StatusCode::NOT_FOUND,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::InvalidToken => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let description = ErrorDescription {
            error: self.to_string(),
            status: status.as_u16(),
        };
        HttpResponse::build(status).json(serde_json::json!(description))
    }
}

#[derive(Debug, Serialize)]
struct ErrorDescription {
    error: String,
    status: u16,
}
