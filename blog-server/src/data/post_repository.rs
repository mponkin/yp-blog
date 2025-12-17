use std::sync::Arc;

use sqlx::PgPool;

use crate::domain::{error::AppError, post::Post};

pub struct PostRepository {
    db_pool: Arc<PgPool>,
}

impl PostRepository {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }

    pub async fn create_post(
        &self,
        title: String,
        content: String,
        author_id: i64,
    ) -> Result<Post, AppError> {
        let query = "
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at";

        sqlx::query_as(query)
            .bind(title)
            .bind(content)
            .bind(author_id)
            .fetch_one(&*self.db_pool)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_post(&self, post_id: i64) -> Result<Option<Post>, AppError> {
        sqlx::query_as(
            "SELECT id, title, content, author_id, created_at, updated_at 
            FROM posts WHERE id = $1",
        )
        .bind(post_id)
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn update_post(
        &self,
        post_id: i64,
        title: String,
        content: String,
        author_id: i64,
    ) -> Result<Post, AppError> {
        let query = "UPDATE posts 
        SET title = $2, content = $3, updated_at = NOW() 
        WHERE id = $1 AND author_id = $4 
        RETURNING id, title, content, author_id, created_at, updated_at";

        sqlx::query_as(query)
            .bind(post_id)
            .bind(title)
            .bind(content)
            .bind(author_id)
            .fetch_one(&*self.db_pool)
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_post(&self, post_id: i64, author_id: i64) -> Result<(), AppError> {
        let query = "DELETE FROM posts
            WHERE id = $1 AND author_id = $2";

        sqlx::query(query)
            .bind(post_id)
            .bind(author_id)
            .execute(&*self.db_pool)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub async fn get_posts(&self, limit: i64, offset: i64) -> Result<Vec<Post>, AppError> {
        let query = "SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2";

        sqlx::query_as(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.db_pool)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_total_posts_count(&self) -> Result<u64, AppError> {
        let query = "SELECT COUNT(*) FROM posts";
        sqlx::query_scalar(query)
            .fetch_one(&*self.db_pool)
            .await
            .map(|count: i64| count as u64)
            .map_err(AppError::from)
    }
}
