#![deny(unreachable_pub)]
#![warn(missing_docs)]

//! WASM blog client

use gloo_net::http::Method;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, Storage};

use crate::{
    dto::{
        LoginRequest, LoginResponse, Post, PostCollection, PostData, RegisterRequest,
        RegisterResponse,
    },
    error::AppError,
};

mod dto;
mod error;

const AUTH_DATA_KEY: &str = "auth_data";

/// Struct for WASM blog client
#[wasm_bindgen]
pub struct BlogApp {
    server_url: String,
    auth_data: Option<AuthData>,
}

#[wasm_bindgen]
impl BlogApp {
    /// Create new client
    #[wasm_bindgen(constructor)]
    pub fn new(server_url: String) -> Result<BlogApp, JsValue> {
        let mut app = BlogApp {
            server_url,
            auth_data: None,
        };

        app.auth_data = app.load_auth_data()?;
        Ok(app)
    }

    /// Register request
    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        let url = format!("{}/auth/register", self.server_url);
        let body = serde_json::json!(RegisterRequest {
            username,
            email,
            password
        });

        let response = Self::request(Method::POST, &url, Some(body), None).await?;
        let auth_response: RegisterResponse = serde_wasm_bindgen::from_value(response)?;
        let auth_data = AuthData::from(auth_response);

        self.save_auth_data(&auth_data)?;
        self.auth_data = Some(auth_data);

        Ok(serde_wasm_bindgen::to_value("register success")?)
    }

    /// Login request
    #[wasm_bindgen]
    pub async fn login(&mut self, username: String, password: String) -> Result<JsValue, JsValue> {
        let url = format!("{}/auth/login", self.server_url);
        let body = serde_json::json!(LoginRequest { username, password });

        let response = Self::request(Method::POST, &url, Some(body), None).await?;
        let login_response: LoginResponse = serde_wasm_bindgen::from_value(response)?;
        let auth_data = AuthData::from(login_response);

        self.save_auth_data(&auth_data)?;
        self.auth_data = Some(auth_data);

        Ok(serde_wasm_bindgen::to_value("log in success")?)
    }

    /// Logout request
    #[wasm_bindgen]
    pub async fn logout(&mut self) -> Result<JsValue, JsValue> {
        self.auth_data = None;
        self.delete_auth_data()?;

        Ok(serde_wasm_bindgen::to_value("log out success")?)
    }

    /// Load posts request
    #[wasm_bindgen]
    pub async fn load_posts(&self, offset: u64, limit: u64) -> Result<JsValue, JsValue> {
        let url = format!("{}/posts?offset={offset}&limit={limit}", self.server_url);

        let response = Self::request(Method::GET, &url, None, None).await?;
        let posts = serde_wasm_bindgen::from_value::<PostCollection>(response)?;
        Ok(serde_wasm_bindgen::to_value(&posts)?)
    }

    /// Create post request
    #[wasm_bindgen]
    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        let url = format!("{}/posts", self.server_url);
        let body = serde_json::json!(PostData { title, content });

        let response = Self::request(Method::POST, &url, Some(body), self.token_opt()).await?;
        let post = serde_wasm_bindgen::from_value::<Post>(response)?;
        Ok(serde_wasm_bindgen::to_value(&post)?)
    }

    /// Update post request
    #[wasm_bindgen]
    pub async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
    ) -> Result<JsValue, JsValue> {
        let url = format!("{}/posts/{}", self.server_url, id);
        let body = serde_json::json!(PostData { title, content });

        let response = Self::request(Method::PUT, &url, Some(body), self.token_opt()).await?;
        let post = serde_wasm_bindgen::from_value::<Post>(response)?;
        Ok(serde_wasm_bindgen::to_value(&post)?)
    }

    /// Delete post request
    #[wasm_bindgen]
    pub async fn delete_post(&self, id: i64) -> Result<JsValue, JsValue> {
        let url = format!("{}/posts/{}", self.server_url, id);
        Self::request(Method::DELETE, &url, None, self.token_opt()).await
    }

    /// Check if user is authenticated
    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> bool {
        self.auth_data.is_some()
    }

    /// Check if post belongs to current user
    #[wasm_bindgen]
    pub fn post_belongs_to_current_user(&self, author_id: i64) -> bool {
        self.auth_data
            .as_ref()
            .is_some_and(|ad| ad.user_id == author_id)
    }

    /// Get post request
    #[wasm_bindgen]
    pub async fn get_post(&self, id: i64) -> Result<JsValue, JsValue> {
        let url = format!("{}/posts/{}", self.server_url, id);
        let response = Self::request(Method::GET, &url, None, None).await?;
        Ok(response)
    }

    fn save_auth_data(&self, auth_data: &AuthData) -> Result<(), AppError> {
        let storage = self.get_local_storage()?;
        let json = serde_json::to_string(auth_data)?;
        storage.set_item(AUTH_DATA_KEY, &json)?;
        Ok(())
    }

    fn delete_auth_data(&self) -> Result<(), AppError> {
        let storage = self.get_local_storage()?;
        storage.remove_item(AUTH_DATA_KEY)?;
        Ok(())
    }

    fn load_auth_data(&self) -> Result<Option<AuthData>, AppError> {
        let storage = self.get_local_storage()?;
        let json_str = if let Some(json_str) = storage.get_item(AUTH_DATA_KEY)? {
            json_str
        } else {
            return Ok(None);
        };
        let data = serde_json::from_str::<AuthData>(&json_str)?;

        Ok(Some(data))
    }

    fn get_local_storage(&self) -> Result<Storage, AppError> {
        let window = web_sys::window().ok_or(AppError::LocalStorageUnavailable)?;
        let local_storage = window
            .local_storage()?
            .ok_or(AppError::LocalStorageUnavailable)?;

        Ok(local_storage)
    }

    fn token_opt(&self) -> Option<&str> {
        match &self.auth_data {
            Some(data) => Some(&data.token),
            None => None,
        }
    }

    async fn request(
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
        token: Option<&str>,
    ) -> Result<JsValue, JsValue> {
        let opts = RequestInit::new();
        opts.set_method(method.as_str());
        opts.set_mode(RequestMode::Cors);

        let headers = web_sys::Headers::new()?;
        headers.append("Content-Type", "application/json")?;

        if let Some(ref token) = token {
            headers.append("Authorization", &format!("Bearer {}", token))?;
        }

        opts.set_headers(&headers);

        if let Some(b) = body {
            let body_str = b.to_string();
            opts.set_body(&JsValue::from_str(&body_str));
        }

        let request = Request::new_with_str_and_init(url, &opts)?;
        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into()?;

        if !resp.ok() {
            return Err(JsValue::from_str(&format!(
                "HTTP error! status: {}",
                resp.status()
            )));
        }

        if resp.status() == 204 {
            Ok(JsValue::UNDEFINED)
        } else {
            JsFuture::from(resp.json()?).await
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthData {
    token: String,
    user_id: i64,
}

impl From<RegisterResponse> for AuthData {
    fn from(value: RegisterResponse) -> Self {
        Self {
            token: value.token,
            user_id: value.user.id,
        }
    }
}

impl From<LoginResponse> for AuthData {
    fn from(value: LoginResponse) -> Self {
        Self {
            token: value.token,
            user_id: value.user.id,
        }
    }
}
