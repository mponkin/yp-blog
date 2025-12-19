use std::{collections::HashMap, time::Duration};

use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::{
    api_client::BlogApiClient,
    blog_client::{Post, PostsCollection},
    error::BlogClientError,
};

pub(crate) struct HttpClient {
    base_url: Url,
    client: Client,
}

impl HttpClient {
    pub(crate) fn new(base_url: &str) -> Result<Self, BlogClientError> {
        let base_url = Url::parse(base_url)?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self { base_url, client })
    }
}

#[async_trait::async_trait]
impl BlogApiClient for HttpClient {
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

        match response.status() {
            StatusCode::CREATED => {
                let user_and_token: UserAndToken = response.json().await?;

                Ok(user_and_token.token)
            }
            StatusCode::CONFLICT => Err(BlogClientError::UserAlreadyExists),
            other => Err(BlogClientError::UnexpectedHttpResponse {
                code: other.as_u16(),
                message: response.text().await?,
            }),
        }
    }

    async fn login(&self, username: String, password: String) -> Result<String, BlogClientError> {
        let url = self.base_url.join("/api/auth/login")?;

        let params = LoginParams { username, password };

        let response = self.client.post(url).json(&params).send().await?;
        match response.status() {
            StatusCode::OK => {
                let user_and_token: UserAndToken = response.json().await?;

                Ok(user_and_token.token)
            }
            StatusCode::UNAUTHORIZED => Err(BlogClientError::InvalidCredentials),
            other => Err(BlogClientError::UnexpectedHttpResponse {
                code: other.as_u16(),
                message: response.text().await?,
            }),
        }
    }

    async fn create_post(
        &self,
        token: &str,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let url = self.base_url.join("/api/posts")?;

        let params = CreatePostParams { title, content };

        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&params)
            .send()
            .await?;
        match response.status() {
            StatusCode::CREATED => {
                let post: Post = response.json().await?;

                Ok(post)
            }
            StatusCode::UNAUTHORIZED => Err(BlogClientError::InvalidToken),
            other => Err(BlogClientError::UnexpectedHttpResponse {
                code: other.as_u16(),
                message: response.text().await?,
            }),
        }
    }

    async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let url = self.base_url.join(format!("/api/posts/{id}").as_str())?;

        let response = self.client.get(url).send().await?;
        match response.status() {
            StatusCode::OK => {
                let post: Post = response.json().await?;

                Ok(post)
            }
            StatusCode::NOT_FOUND => Err(BlogClientError::NotFound),
            other => Err(BlogClientError::UnexpectedHttpResponse {
                code: other.as_u16(),
                message: response.text().await?,
            }),
        }
    }

    async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let url = self.base_url.join(format!("/api/posts/{id}").as_str())?;

        let params = UpdatePostParams { title, content };

        let response = self
            .client
            .put(url)
            .bearer_auth(token)
            .json(&params)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let post: Post = response.json().await?;

                Ok(post)
            }
            StatusCode::UNAUTHORIZED => Err(BlogClientError::InvalidToken),
            StatusCode::FORBIDDEN => Err(BlogClientError::Forbidden),
            other => Err(BlogClientError::UnexpectedHttpResponse {
                code: other.as_u16(),
                message: response.text().await?,
            }),
        }
    }

    async fn delete_post(&self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let url = self.base_url.join(format!("/api/posts/{id}").as_str())?;

        let response = self.client.delete(url).bearer_auth(token).send().await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::UNAUTHORIZED => Err(BlogClientError::InvalidToken),
            StatusCode::FORBIDDEN => Err(BlogClientError::Forbidden),
            other => Err(BlogClientError::UnexpectedHttpResponse {
                code: other.as_u16(),
                message: response.text().await?,
            }),
        }
    }

    async fn get_posts(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<PostsCollection, BlogClientError> {
        let url = self.base_url.join("/api/posts")?;

        let mut query = HashMap::new();
        if let Some(limit) = limit {
            query.insert("limit", limit);
        }

        if let Some(offset) = offset {
            query.insert("offset", offset);
        }

        let response = self.client.get(url).query(&query).send().await?;

        match response.status() {
            StatusCode::OK => {
                let posts_response: PostsCollection = response.json().await?;

                Ok(posts_response)
            }
            other => Err(BlogClientError::UnexpectedHttpResponse {
                code: other.as_u16(),
                message: response.text().await?,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
struct CreateUserParams {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginParams {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct UserAndToken {
    token: String,
}

#[derive(Debug, Serialize)]
struct CreatePostParams {
    title: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct UpdatePostParams {
    title: String,
    content: String,
}
