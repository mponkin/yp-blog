//! Blog client using GRPC protocol

use std::time::Duration;

use blog_grpc_api::{
    CreatePostRequest, DeletePostRequest, GetPostRequest, GetPostsRequest, LoginRequest,
    RegisterRequest, UpdatePostRequest, blog_service_client::BlogServiceClient,
};
use chrono::{DateTime, Utc};
use tonic::{
    IntoRequest, Request,
    metadata::MetadataValue,
    transport::{Channel, Endpoint},
};

use crate::{
    api_client::BlogApiClient,
    blog_client::{Post, PostsCollection},
    error::BlogClientError,
};

/// GRPC client for blog-server
pub(crate) struct GrpcClient {
    client: BlogServiceClient<Channel>,
}

impl GrpcClient {
    pub(crate) async fn new(url: String) -> Result<Self, BlogClientError> {
        let endpoint = Endpoint::from_shared(url)?;
        let channel = endpoint
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?;
        let client = BlogServiceClient::new(channel);
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl BlogApiClient for GrpcClient {
    async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<String, BlogClientError> {
        let mut client = self.client.clone();

        let response = client
            .register(
                RegisterRequest {
                    username,
                    email,
                    password,
                }
                .into_request(),
            )
            .await?
            .into_inner();

        Ok(response.token)
    }

    async fn login(&self, username: String, password: String) -> Result<String, BlogClientError> {
        let mut client = self.client.clone();

        let response = client
            .login(LoginRequest { username, password }.into_request())
            .await?
            .into_inner();

        Ok(response.token)
    }

    async fn create_post(
        &self,
        token: &str,
        title: String,
        content: String,
    ) -> Result<crate::blog_client::Post, BlogClientError> {
        let mut client = self.client.clone();

        let response = client
            .create_post(
                CreatePostRequest { title, content }
                    .into_request()
                    .with_token_auth(token)?,
            )
            .await?
            .into_inner();

        let post = response
            .post
            .ok_or_else(|| BlogClientError::GrpcFieldNotSet(String::from("post")))?;

        into_domain_post(post)
    }

    async fn get_post(&self, id: i64) -> Result<crate::blog_client::Post, BlogClientError> {
        let mut client = self.client.clone();

        let response = client
            .get_post(GetPostRequest { post_id: id }.into_request())
            .await?
            .into_inner();

        let post = response
            .post
            .ok_or_else(|| BlogClientError::GrpcFieldNotSet(String::from("post")))?;

        into_domain_post(post)
    }

    async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: String,
        content: String,
    ) -> Result<crate::blog_client::Post, BlogClientError> {
        let mut client = self.client.clone();

        let response = client
            .update_post(
                UpdatePostRequest {
                    post_id: id,
                    title,
                    content,
                }
                .into_request()
                .with_token_auth(token)?,
            )
            .await?
            .into_inner();

        let post = response
            .post
            .ok_or_else(|| BlogClientError::GrpcFieldNotSet(String::from("post")))?;

        into_domain_post(post)
    }

    async fn delete_post(&self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let mut client = self.client.clone();

        client
            .delete_post(
                DeletePostRequest { post_id: id }
                    .into_request()
                    .with_token_auth(token)?,
            )
            .await?
            .into_inner();

        Ok(())
    }

    async fn get_posts(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<PostsCollection, BlogClientError> {
        let mut client = self.client.clone();

        let response = client
            .get_posts(
                GetPostsRequest {
                    limit: limit.map(|l| l as i64),
                    offset: offset.map(|o| o as i64),
                }
                .into_request(),
            )
            .await?
            .into_inner();

        Ok(PostsCollection {
            posts: response
                .posts
                .into_iter()
                .map(into_domain_post)
                .collect::<Result<Vec<_>, BlogClientError>>()?,
            limit: response.limit as u64,
            offset: response.offset as u64,
            total_posts: response.total_posts_count as u64,
        })
    }
}

fn into_domain_post(post: blog_grpc_api::Post) -> Result<Post, BlogClientError> {
    Ok(Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: timestamp_to_datetime(post.created_at)?,
        updated_at: timestamp_to_datetime(post.updated_at)?,
    })
}

fn timestamp_to_datetime(ts: i64) -> Result<DateTime<Utc>, BlogClientError> {
    DateTime::from_timestamp_millis(ts).ok_or_else(|| BlogClientError::IncorrectTimestamp(ts))
}

trait WithTokenAuth {
    fn with_token_auth(self, token: &str) -> Result<Self, BlogClientError>
    where
        Self: Sized;
}

impl<T> WithTokenAuth for Request<T> {
    fn with_token_auth(mut self, token: &str) -> Result<Self, BlogClientError>
    where
        Self: Sized,
    {
        let meta = MetadataValue::try_from(format!("Bearer {token}"))?;
        self.metadata_mut().insert("authorization", meta);
        Ok(self)
    }
}
