//! Module containing description of blog client interface and related structures

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::BlogClientError;

/// Trait for blog client interface
#[async_trait::async_trait]
pub trait BlogClient {
    /// Sets JWT token
    ///
    /// # Arguments
    /// * `token` - JWT token, returned from `register` or `login` functions
    fn set_token(&mut self, token: String);

    /// Returns stored JWT token if it is set
    fn get_token(&self) -> Option<String>;

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
    async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<String, BlogClientError>;

    /// Login existing user
    ///
    /// # Arguments
    ///
    /// * `username` - user name
    /// * `password` - user password
    ///
    /// # Returns Ok(String) with JWT token if user is logged in successfully
    /// # Returns Err(BlogClientError) otherwise
    async fn login(&self, username: String, password: String) -> Result<String, BlogClientError>;

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
    async fn create_post(&self, title: String, content: String) -> Result<Post, BlogClientError>;

    /// Gets a post by id
    ///
    /// # Arguments
    ///
    /// * `id` - requested post id
    ///
    /// # Returns Ok(Post) contatining the requested post if the post fetched successfully
    /// # Returns Err(BlogClientError) otherwise
    async fn get_post(&self, id: i64) -> Result<Post, BlogClientError>;

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
    async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError>;

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
    async fn delete_post(&self, id: i64) -> Result<(), BlogClientError>;

    /// Gets list of posts
    ///
    /// # Arguments
    ///
    /// * `limit` - optional number of posts to fetch
    /// * `offset` - optional offset of first fetched post
    ///
    /// # Returns Ok(PostsResponse) if fetched successfully
    /// # Returns Err(BlogClientError) otherwise
    async fn get_posts(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<PostsResponse, BlogClientError>;
}

/// Response for list of posts
#[derive(Debug, Deserialize)]
pub struct PostsResponse {
    /// List of posts
    pub posts: Vec<Post>,
    /// Number of fetched posts
    pub limit: u64,
    /// Offset of first fetched post
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

/// Trait for providing token for operations that requires token auth
pub(crate) trait RequireToken {
    /// # Returns
    /// Ok(token) if token can be provided
    /// Err(TokenNotSet::TokenNotSet) otherwise
    fn require_token(&self) -> Result<&str, BlogClientError>;
}
