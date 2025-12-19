use enum_dispatch::enum_dispatch;

use crate::{
    blog_client::{Post, PostsCollection},
    error::BlogClientError,
    grpc_client::GrpcClient,
    http_client::HttpClient,
};

/// Trait for blog client interface
#[async_trait::async_trait]
#[enum_dispatch(ClientType)]
pub(crate) trait BlogApiClient {
    async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<String, BlogClientError>;

    async fn login(&self, username: String, password: String) -> Result<String, BlogClientError>;

    async fn create_post(
        &self,
        token: &str,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError>;

    async fn get_post(&self, id: i64) -> Result<Post, BlogClientError>;

    async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError>;

    async fn delete_post(&self, token: &str, id: i64) -> Result<(), BlogClientError>;

    async fn get_posts(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<PostsCollection, BlogClientError>;
}

#[enum_dispatch]
pub(crate) enum ClientType {
    HttpClient,
    GrpcClient,
}
