use crate::{
    data::post_repository::PostRepository,
    domain::{error::AppError, post::Post},
};

pub struct BlogService {
    post_repo: PostRepository,
}

impl BlogService {
    pub fn new(post_repo: PostRepository) -> Self {
        Self { post_repo }
    }

    pub async fn create_post(
        &self,
        title: String,
        content: String,
        author_id: i64,
    ) -> Result<Post, AppError> {
        self.post_repo.create_post(title, content, author_id).await
    }

    pub async fn get_post(&self, post_id: i64) -> Result<Post, AppError> {
        match self.post_repo.get_post(post_id).await {
            Ok(Some(post)) => Ok(post),
            Ok(None) => Err(AppError::PostNotFound),
            Err(e) => Err(e),
        }
    }

    pub async fn update_post(
        &self,
        post_id: i64,
        title: String,
        content: String,
        user_id: i64,
    ) -> Result<Post, AppError> {
        let post = self.get_post(post_id).await?;
        if post.author_id != user_id {
            return Err(AppError::Forbidden);
        }

        self.post_repo
            .update_post(post_id, title, content, user_id)
            .await
    }

    pub async fn delete_post(&self, post_id: i64, user_id: i64) -> Result<(), AppError> {
        let post = self.get_post(post_id).await?;
        if post.author_id != user_id {
            return Err(AppError::Forbidden);
        }

        self.post_repo.delete_post(post_id, user_id).await
    }

    pub async fn get_posts(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, u64), AppError> {
        let posts = self.post_repo.get_posts(limit, offset).await?;
        let total_posts = self.post_repo.get_total_posts_count().await?;

        Ok((posts, total_posts))
    }
}
