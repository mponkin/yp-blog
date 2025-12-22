use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct RegisterRequest {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterResponse {
    pub(crate) token: String,
    pub(crate) user: User,
}

#[derive(Debug, Deserialize)]
pub(crate) struct User {
    pub(crate) id: i64,
    // other fields are omitted
}

#[derive(Debug, Serialize)]
pub(crate) struct LoginRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginResponse {
    pub(crate) token: String,
    pub(crate) user: User,
}

#[derive(Debug, Serialize)]
pub(crate) struct PostData {
    pub(crate) title: String,
    pub(crate) content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Post {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) author_id: i64,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PostCollection {
    pub(crate) posts: Vec<Post>,
    pub(crate) limit: u64,
    pub(crate) offset: u64,
    pub(crate) total_posts: u64,
}
