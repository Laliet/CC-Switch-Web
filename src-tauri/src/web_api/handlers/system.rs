#![cfg(feature = "web-server")]

use std::sync::Arc;

use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use super::{ApiError, ApiResult};

/// Stub handler for tray updates in web mode.
pub async fn update_tray() -> ApiResult<bool> {
    Ok(Json(true))
}

#[derive(Deserialize)]
pub struct OpenExternalPayload {
    pub url: String,
}

/// Validate and acknowledge external URL open request in web mode.
/// 实际浏览器打开操作应由前端完成，这里仅作校验避免 404。
pub async fn open_external(Json(payload): Json<OpenExternalPayload>) -> ApiResult<bool> {
    let parsed = url::Url::parse(&payload.url)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let scheme = parsed.scheme().to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "Unsupported URL scheme",
        ));
    }
    Ok(Json(true))
}

/// Return the current CSRF token for the session.
/// This endpoint requires Basic Auth but does NOT require CSRF token (it's a GET request).
pub async fn get_csrf_token(Extension(csrf): Extension<Option<Arc<String>>>) -> impl IntoResponse {
    match csrf {
        Some(token) => Json(serde_json::json!({ "csrfToken": token.as_str() })),
        None => Json(serde_json::json!({ "csrfToken": null })),
    }
}
