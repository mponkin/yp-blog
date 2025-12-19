use std::sync::Arc;

use blog_grpc_api::{
    AuthResponse, CreatePostRequest, DeletePostRequest, GetPostRequest, GetPostsRequest,
    GetPostsResponse, LoginRequest, PostResponse, RegisterRequest, UpdatePostRequest,
    blog_service_server::BlogService,
};
use tonic::async_trait;

use crate::{
    application::auth_service::AuthService,
    domain::{error::AppError, post::Post},
    infrastructure::jwt::JwtService,
};

pub(crate) struct GrpcService {
    auth_service: Arc<AuthService>,
    posts_service: Arc<crate::application::blog_service::BlogService>,
    jwt_service: Arc<JwtService>,
}

impl GrpcService {
    pub(crate) fn new(
        auth_service: Arc<AuthService>,
        posts_service: Arc<crate::application::blog_service::BlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            posts_service,
            jwt_service,
        }
    }

    fn get_user_id<T>(&self, request: &tonic::Request<T>) -> Result<i64, AppError> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|s| s.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or(AppError::InvalidToken)?;

        Ok(self.jwt_service.verify_token(token)?.user_id)
    }
}

#[async_trait]
impl BlogService for GrpcService {
    async fn register(
        &self,
        request: tonic::Request<RegisterRequest>,
    ) -> Result<tonic::Response<AuthResponse>, tonic::Status> {
        let params = request.into_inner();
        let token = self
            .auth_service
            .register(params.username, params.email, params.password)
            .await
            .map(|user_and_token| user_and_token.token)?;

        Ok(AuthResponse { token }.into())
    }
    async fn login(
        &self,
        request: tonic::Request<LoginRequest>,
    ) -> Result<tonic::Response<AuthResponse>, tonic::Status> {
        let params = request.into_inner();
        let token = self
            .auth_service
            .login(params.username, params.password)
            .await
            .map(|user_and_token| user_and_token.token)?;

        Ok(AuthResponse { token }.into())
    }
    async fn create_post(
        &self,
        request: tonic::Request<CreatePostRequest>,
    ) -> Result<tonic::Response<PostResponse>, tonic::Status> {
        let user_id = self.get_user_id(&request)?;
        let params = request.into_inner();
        let post = self
            .posts_service
            .create_post(params.title, params.content, user_id)
            .await?;
        Ok(to_post_response(post).into())
    }
    async fn get_post(
        &self,
        request: tonic::Request<GetPostRequest>,
    ) -> Result<tonic::Response<PostResponse>, tonic::Status> {
        let params = request.into_inner();
        let post = self.posts_service.get_post(params.post_id).await?;
        Ok(to_post_response(post).into())
    }
    async fn update_post(
        &self,
        request: tonic::Request<UpdatePostRequest>,
    ) -> Result<tonic::Response<PostResponse>, tonic::Status> {
        let user_id = self.get_user_id(&request)?;
        let params = request.into_inner();
        let post = self
            .posts_service
            .update_post(params.post_id, params.title, params.content, user_id)
            .await?;
        Ok(to_post_response(post).into())
    }
    async fn delete_post(
        &self,
        request: tonic::Request<DeletePostRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let user_id = self.get_user_id(&request)?;
        let params = request.into_inner();
        self.posts_service
            .delete_post(params.post_id, user_id)
            .await?;
        Ok(().into())
    }
    async fn get_posts(
        &self,
        request: tonic::Request<GetPostsRequest>,
    ) -> Result<tonic::Response<GetPostsResponse>, tonic::Status> {
        let params = request.into_inner();
        let limit = params.limit.unwrap_or(10);
        let offset = params.offset.unwrap_or(0);
        let (posts, total_posts_count) = self.posts_service.get_posts(limit, offset).await?;
        Ok(GetPostsResponse {
            posts: posts.into_iter().map(to_grpc_post).collect(),
            limit,
            offset,
            total_posts_count: total_posts_count as i64,
        }
        .into())
    }
}

impl From<AppError> for tonic::Status {
    fn from(value: AppError) -> Self {
        match value {
            AppError::UserNotFound { .. } => tonic::Status::not_found(value.to_string()),
            AppError::UserAlreadyExists => tonic::Status::already_exists(value.to_string()),
            AppError::InvalidCredentials => tonic::Status::unauthenticated(value.to_string()),
            AppError::PostNotFound => tonic::Status::not_found(value.to_string()),
            AppError::Forbidden => tonic::Status::permission_denied(value.to_string()),
            AppError::InvalidToken => tonic::Status::unauthenticated(value.to_string()),
            value => tonic::Status::internal(value.to_string()),
        }
    }
}

fn to_grpc_post(post: Post) -> blog_grpc_api::Post {
    blog_grpc_api::Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: post.created_at.timestamp_millis(),
        updated_at: post.updated_at.timestamp_millis(),
    }
}

fn to_post_response(post: Post) -> PostResponse {
    PostResponse {
        post: Some(to_grpc_post(post)),
    }
}
