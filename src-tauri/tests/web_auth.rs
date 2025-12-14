#![cfg(feature = "web-server")]

use std::sync::{Arc, RwLock};

use axum::{
    body::Body,
    http::{header::AUTHORIZATION, HeaderValue, Method, Request, StatusCode},
};
use base64::Engine;
use cc_switch_lib::{web_api, AppState, MultiAppConfig};
use serial_test::serial;
use tower::ServiceExt;

#[path = "support.rs"]
mod support;
use support::{ensure_test_home, reset_test_fs, test_mutex};

fn basic_auth_header(user: &str, password: &str) -> HeaderValue {
    let raw = format!("{user}:{password}");
    let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
    HeaderValue::from_str(&format!("Basic {encoded}")).expect("basic auth header")
}

fn make_app(password: &str, csrf: &str) -> axum::Router {
    std::env::set_var("WEB_CSRF_TOKEN", csrf);
    let state = Arc::new(AppState {
        config: RwLock::new(MultiAppConfig::default()),
    });
    web_api::create_router(state, password.to_string())
}

async fn dispatch(app: axum::Router, request: Request<Body>) -> axum::response::Response {
    app.oneshot(request).await.expect("router response")
}

#[tokio::test]
#[serial]
async fn test_basic_auth_valid() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let app = make_app("password", "csrf-token");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/config/app/path")
        .header(AUTHORIZATION, basic_auth_header("admin", "password"))
        .body(Body::empty())
        .unwrap();

    let res = dispatch(app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn test_basic_auth_invalid_password() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let app = make_app("password", "csrf-token");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/config/app/path")
        .header(AUTHORIZATION, basic_auth_header("admin", "wrong"))
        .body(Body::empty())
        .unwrap();

    let res = dispatch(app, req).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_basic_auth_invalid_user() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let app = make_app("password", "csrf-token");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/config/app/path")
        .header(AUTHORIZATION, basic_auth_header("not-admin", "password"))
        .body(Body::empty())
        .unwrap();

    let res = dispatch(app, req).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_basic_auth_missing() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let app = make_app("password", "csrf-token");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/config/app/path")
        .body(Body::empty())
        .unwrap();

    let res = dispatch(app, req).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_csrf_required_for_post() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let app = make_app("password", "csrf-token");

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/tray/update")
        .header(AUTHORIZATION, basic_auth_header("admin", "password"))
        .body(Body::empty())
        .unwrap();

    let res = dispatch(app, req).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[serial]
async fn test_csrf_not_required_for_get() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let app = make_app("password", "csrf-token");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/config/app/path")
        .header(AUTHORIZATION, basic_auth_header("admin", "password"))
        .body(Body::empty())
        .unwrap();

    let res = dispatch(app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn test_csrf_invalid_token() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let app = make_app("password", "csrf-token");

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/tray/update")
        .header(AUTHORIZATION, basic_auth_header("admin", "password"))
        .header("x-csrf-token", HeaderValue::from_static("wrong-token"))
        .body(Body::empty())
        .unwrap();

    let res = dispatch(app, req).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[serial]
async fn test_security_headers_present() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let app = make_app("password", "csrf-token");

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/tray/update")
        .header(AUTHORIZATION, basic_auth_header("admin", "password"))
        .header("x-csrf-token", HeaderValue::from_static("csrf-token"))
        .body(Body::empty())
        .unwrap();

    let res = dispatch(app, req).await;
    assert_eq!(res.status(), StatusCode::OK);

    let headers = res.headers();
    assert_eq!(
        headers
            .get("strict-transport-security")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        "max-age=31536000; includeSubDomains"
    );
    assert_eq!(
        headers
            .get("x-frame-options")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        "DENY"
    );
    assert_eq!(
        headers
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        "nosniff"
    );
    assert_eq!(
        headers
            .get("referrer-policy")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        "no-referrer"
    );
}

