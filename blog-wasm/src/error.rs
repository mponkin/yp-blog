use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Can't access local storage")]
    LocalStorageUnavailable,
    #[error("Error converting object to/from JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("JsValue: {}", 0.to_string())]
    JsValue(wasm_bindgen::JsValue),
}

impl From<JsValue> for AppError {
    fn from(value: JsValue) -> Self {
        AppError::JsValue(value)
    }
}

impl From<AppError> for JsValue {
    fn from(value: AppError) -> Self {
        match value {
            AppError::JsValue(js_value) => js_value,
            other => JsValue::from_str(other.to_string().as_str()),
        }
    }
}
