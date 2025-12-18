//! Blog client using GRPC protocol
#![allow(unused)]

use tonic::include_proto;
include_proto!("blog");

use crate::{
    blog_client::{BlogClient, Post, PostsResponse},
    error::BlogClientError,
};

/// GRPC client for blog-server
pub(crate) struct GrpcClient {}

#[async_trait::async_trait]
impl BlogClient for GrpcClient {
    fn set_token(&mut self, token: String) {
        todo!()
    }

    fn get_token(&self) -> Option<String> {
        todo!()
    }

    async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<String, BlogClientError> {
        todo!()
    }

    async fn login(&self, username: String, password: String) -> Result<String, BlogClientError> {
        todo!()
    }

    async fn create_post(
        &self,
        title: String,
        content: String,
    ) -> Result<crate::blog_client::Post, BlogClientError> {
        todo!()
    }

    async fn get_post(&self, id: i64) -> Result<crate::blog_client::Post, BlogClientError> {
        todo!()
    }

    async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<crate::blog_client::Post, BlogClientError> {
        todo!()
    }

    async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        todo!()
    }

    async fn get_posts(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<PostsResponse, BlogClientError> {
        todo!()
    }
}
