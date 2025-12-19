//! Module containing description of blog client interface and related structures

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    Transport,
    api_client::{BlogApiClient, ClientType},
    error::BlogClientError,
    grpc_client::GrpcClient,
    http_client::HttpClient,
};

/// Client for blog backend interation
pub struct BlogClient {
    inner: ClientType,
    token: Option<String>,
}

impl BlogClient {
    /// Creates client with inner api client based on transport parameter
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let inner = match transport {
            Transport::Http(url) => ClientType::HttpClient(HttpClient::new(url.as_str())?),
            Transport::Grpc(url) => ClientType::GrpcClient(GrpcClient::new(url).await?),
        };

        Ok(Self { inner, token: None })
    }

    /// Sets JWT token
    ///
    /// # Arguments
    /// * `token` - JWT token, returned from `register` or `login` functions
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token)
    }

    /// Returns stored JWT token if it is set
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Register a new user
    ///
    /// # Arguments
    ///
    /// * `username` - user name
    /// * `email` - user email
    /// * `password` - user password
    ///
    /// # Returns Ok(String) with JWT token if user is registered successfully
    /// # Returns Err(BlogClientError) otherwise
    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<String, BlogClientError> {
        self.inner.register(username, email, password).await
    }

    /// Login existing user
    ///
    /// # Arguments
    ///
    /// * `username` - user name
    /// * `password` - user password
    ///
    /// # Returns Ok(String) with JWT token if user is logged in successfully
    /// # Returns Err(BlogClientError) otherwise
    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<String, BlogClientError> {
        self.inner.login(username, password).await
    }

    /// Creates a new post
    ///
    /// requires token to be set through `set_token`
    ///
    /// # Arguments
    ///
    /// * `title` - new post title
    /// * `content` - new post content
    ///
    /// # Returns Ok(Post) with created post if it is created successfully
    /// # Returns Err(BlogClientError) otherwise
    pub async fn create_post(
        &self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        self.inner
            .create_post(self.require_token()?, title, content)
            .await
    }

    /// Gets a post by id
    ///
    /// # Arguments
    ///
    /// * `id` - requested post id
    ///
    /// # Returns Ok(Post) contatining the requested post if the post fetched successfully
    /// # Returns Err(BlogClientError) otherwise
    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        self.inner.get_post(id).await
    }

    /// Updates the post with given id
    ///
    /// requires token to be set through `set_token`
    /// only original author can edit the post
    ///
    /// # Arguments
    ///
    /// * `id` - requested post id
    /// * `title` - new post title
    /// * `content` - new post content
    ///
    /// # Returns Ok(Post) with the updated post if it is updated successfully
    /// # Returns Err(BlogClientError) otherwise
    pub async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        self.inner
            .update_post(self.require_token()?, id, title, content)
            .await
    }

    /// Deletes the post with given id
    ///
    /// requires token to be set through `set_token`
    /// only original author can delete the post
    ///
    /// # Arguments
    ///
    /// * `id` -  post id
    ///
    /// # Returns Ok(()) if it is deleted successfully
    /// # Returns Err(BlogClientError) otherwise
    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        self.inner.delete_post(self.require_token()?, id).await
    }

    /// Gets list of posts
    ///
    /// # Arguments
    ///
    /// * `limit` - optional number of posts to fetch
    /// * `offset` - optional offset of first fetched post
    ///
    /// # Returns Ok(PostsCollection) if fetched successfully
    /// # Returns Err(BlogClientError) otherwise
    pub async fn get_posts(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<PostsCollection, BlogClientError> {
        self.inner.get_posts(limit, offset).await
    }

    fn require_token(&self) -> Result<&str, BlogClientError> {
        self.get_token().ok_or(BlogClientError::TokenNotSet)
    }
}

/// Response for list of posts
#[derive(Debug, Deserialize)]
pub struct PostsCollection {
    /// List of posts
    pub posts: Vec<Post>,
    /// Number of requested posts
    pub limit: u64,
    /// Offset of first requested post
    pub offset: u64,
    /// Total count of posts available to fetch
    pub total_posts: u64,
}

/// Post structure
#[derive(Debug, Deserialize)]
pub struct Post {
    /// post id
    pub id: i64,
    /// post title
    pub title: String,
    /// post content
    pub content: String,
    /// user id of post author
    pub author_id: i64,
    /// when post was created
    pub created_at: DateTime<Utc>,
    /// when post was updated last time
    pub updated_at: DateTime<Utc>,
}
