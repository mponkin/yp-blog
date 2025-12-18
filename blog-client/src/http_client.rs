use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::{
    blog_client::{BlogClient, Post, PostsResponse, RequireToken},
    error::BlogClientError,
};

pub(crate) struct HttpClient {
    base_url: Url,
    client: Client,
    token: Option<String>,
}

impl HttpClient {
    pub(crate) fn new(base_url: &str) -> Result<Self, BlogClientError> {
        let base_url = Url::parse(base_url)?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            base_url,
            client,
            token: None,
        })
    }
}

#[async_trait::async_trait]
impl BlogClient for HttpClient {
    fn set_token(&mut self, token: String) {
        self.token = Some(token)
    }

    fn get_token(&self) -> Option<String> {
        self.token.clone()
    }

    async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<String, BlogClientError> {
        let url = self.base_url.join("/api/auth/register")?;

        let params = CreateUserParams {
            username,
            email,
            password,
        };

        let response = self.client.post(url).json(&params).send().await?;
        let user_and_token: UserAndToken = response.json().await?;

        Ok(user_and_token.token)
    }

    async fn login(&self, username: String, password: String) -> Result<String, BlogClientError> {
        let url = self.base_url.join("/api/auth/login")?;

        let params = LoginParams { username, password };

        let response = self.client.post(url).json(&params).send().await?;
        let user_and_token: UserAndToken = response.json().await?;

        Ok(user_and_token.token)
    }

    async fn create_post(&self, title: String, content: String) -> Result<Post, BlogClientError> {
        let url = self.base_url.join("/api/posts")?;

        let params = CreatePostParams { title, content };

        let response = self
            .client
            .post(url)
            .bearer_auth(self.require_token()?)
            .json(&params)
            .send()
            .await?;
        let post: Post = response.json().await?;

        Ok(post)
    }

    async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let url = self.base_url.join(format!("/api/posts/{id}").as_str())?;

        let response = self.client.get(url).send().await?;
        let post: Post = response.json().await?;

        Ok(post)
    }

    async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let url = self.base_url.join(format!("/api/posts/{id}").as_str())?;

        let params = UpdatePostParams { title, content };

        let response = self
            .client
            .put(url)
            .bearer_auth(self.require_token()?)
            .json(&params)
            .send()
            .await?;
        let post: Post = response.json().await?;

        Ok(post)
    }

    async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        let url = self.base_url.join(format!("/api/posts/{id}").as_str())?;

        self.client
            .delete(url)
            .bearer_auth(self.require_token()?)
            .send()
            .await?;

        Ok(())
    }

    async fn get_posts(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<PostsResponse, BlogClientError> {
        let url = self.base_url.join("/api/posts")?;

        let mut query = HashMap::new();
        if let Some(limit) = limit {
            query.insert("limit", limit);
        }

        if let Some(offset) = offset {
            query.insert("offset", offset);
        }

        let response = self.client.get(url).query(&query).send().await?;
        let posts_response: PostsResponse = response.json().await?;

        Ok(posts_response)
    }
}

impl RequireToken for HttpClient {
    fn require_token(&self) -> Result<&str, BlogClientError> {
        match &self.token {
            Some(t) => Ok(t.as_str()),
            None => Err(BlogClientError::TokenNotSet),
        }
    }
}

#[derive(Debug, Deserialize)]
struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct CreateUserParams {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
struct LoginParams {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
struct UserAndToken {
    pub user: User,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePostParams {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct UpdatePostParams {
    pub title: String,
    pub content: String,
}
