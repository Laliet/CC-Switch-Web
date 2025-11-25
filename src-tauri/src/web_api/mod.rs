#![cfg(feature = "web-server")]

use std::{env, fs, path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    extract::Path,
    http::{
        header::{
            self, ACCEPT, AUTHORIZATION, CONTENT_TYPE, STRICT_TRANSPORT_SECURITY, WWW_AUTHENTICATE,
        },
        HeaderValue, Method, Request, StatusCode,
    },
    middleware,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::Engine;
use mime_guess::mime;
use rust_embed::RustEmbed;
use std::sync::Arc as StdArc;
use tower_http::{cors::CorsLayer, validate_request::ValidateRequestHeaderLayer};

use crate::{
    config::{atomic_write, get_home_dir},
    store::AppState,
};

pub mod handlers;
pub mod routes;

/// Shared application state for the web server.
pub type SharedState = Arc<AppState>;

#[derive(RustEmbed)]
#[folder = "../dist-web"]
struct WebAssets;

#[derive(Clone)]
struct WebTokens {
    api_token: String,
    csrf_token: String,
}

/// Serve embedded static assets with index.html fallback for SPA routes.
async fn serve_static(path: Option<Path<String>>, tokens: Arc<WebTokens>) -> impl IntoResponse {
    let requested_path = path.map(|Path(p)| p).unwrap_or_default();
    let requested_path = requested_path.trim_start_matches('/');
    let target_path = if requested_path.is_empty() {
        "index.html"
    } else {
        requested_path
    };

    // Try the requested file first; fall back to index.html so SPA routes resolve client-side.
    let (asset, served_path) = match WebAssets::get(target_path) {
        Some(content) => (content, target_path),
        None => match WebAssets::get("index.html") {
            Some(content) => (content, "index.html"),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };

    let mime = mime_guess::from_path(served_path).first_or(mime::APPLICATION_OCTET_STREAM);
    let mut content = asset.data.into_owned();

    if served_path == "index.html" {
        if let Ok(mut html) = String::from_utf8(content.clone()) {
            let injection = format!(
                r#"<script>
window.__CC_SWITCH_TOKENS__ = {{
  apiToken: "{api}",
  csrfToken: "{csrf}"
}};
</script>"#,
                api = tokens.api_token,
                csrf = tokens.csrf_token
            );
            if let Some(pos) = html.find("</head>") {
                html.insert_str(pos, &injection);
            } else {
                html.push_str(&injection);
            }
            content = html.into_bytes();
        }
    }

    let body = Body::from(content);

    let mut response = Response::new(body);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );

    response
}

fn cors_layer() -> CorsLayer {
    // Production-safe CORS defaults. Enable explicitly via env when cross-origin access is needed.
    let allow_origins = env::var("CORS_ALLOW_ORIGINS").ok();
    let allow_credentials = env::var("CORS_ALLOW_CREDENTIALS")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false);

    let mut layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE]);

    match allow_origins.as_deref() {
        Some("*") => {
            // 显式禁止生产中的通配符，防止意外放开
            log::warn!("CORS_ALLOW_ORIGINS='*' 已被忽略，请使用逗号分隔的白名单");
            return layer;
        }
        Some(list) => {
            let origins: Vec<HeaderValue> = list
                .split(',')
                .filter_map(|entry| {
                    let trimmed = entry.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        HeaderValue::from_str(trimmed).ok()
                    }
                })
                .collect();

            if origins.is_empty() {
                return layer;
            }
            layer = layer.allow_origin(origins);
        }
        None => {
            // No CORS allow-list provided -> rely on same-origin; do not loosen automatically.
            return layer;
        }
    }

    if allow_credentials {
        layer = layer.allow_credentials(true);
    }

    layer
}

/// Construct the axum router with all API routes and middleware.
pub fn create_router(state: SharedState, password: String) -> Router {
    let tokens = Arc::new(load_or_generate_tokens());

    let hsts_enabled = env::var("ENABLE_HSTS")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(true);

    let auth_validator = AuthValidator::new(
        password,
        Some(tokens.api_token.clone()),
        Some(tokens.csrf_token.clone()),
    );

    let router = routes::create_router(state)
        .layer(ValidateRequestHeaderLayer::custom(auth_validator))
        .layer(middleware::from_fn({
            let hsts_enabled = hsts_enabled;
            move |req, next| add_hsts_header(hsts_enabled, req, next)
        }));

    // Only apply CORS when explicitly configured via env; default to same-origin.
    let router = if env::var("CORS_ALLOW_ORIGINS").is_ok() {
        router.layer(cors_layer())
    } else {
        router
    };

    Router::new()
        .nest("/api", router)
        .route(
            "/",
            get({
                let tokens = tokens.clone();
                move |path| serve_static(path, tokens.clone())
            }),
        )
        .route(
            "/*path",
            get({
                let tokens = tokens.clone();
                move |path| serve_static(path, tokens.clone())
            }),
        )
}

#[derive(Clone)]
struct AuthValidator {
    basic_user: StdArc<String>,
    basic_pass: StdArc<String>,
    bearer: Option<StdArc<String>>,
    csrf_token: Option<StdArc<String>>,
}

impl AuthValidator {
    fn new(password: String, bearer: Option<String>, csrf_token: Option<String>) -> Self {
        Self {
            basic_user: StdArc::new("admin".to_string()),
            basic_pass: StdArc::new(password),
            bearer: bearer.map(StdArc::new),
            csrf_token: csrf_token.map(StdArc::new),
        }
    }

    fn is_authorized(&self, auth_value: &str) -> bool {
        if let Some(token) = self.bearer.as_ref() {
            if auth_value
                .trim_start()
                .to_ascii_lowercase()
                .starts_with("bearer")
            {
                return auth_value
                    .split_once(' ')
                    .map(|(_, v)| v == token.as_ref())
                    .unwrap_or(false);
            }
        }

        if let Some(raw) = auth_value.strip_prefix("Basic ") {
            if let Ok(decoded) =
                base64::engine::general_purpose::STANDARD.decode(raw.trim().as_bytes())
            {
                if let Ok(s) = String::from_utf8(decoded) {
                    if let Some((user, pass)) = s.split_once(':') {
                        return user == self.basic_user.as_str()
                            && pass == self.basic_pass.as_str();
                    }
                }
            }
        }

        false
    }

    fn unauthorized() -> Response {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(
                WWW_AUTHENTICATE,
                HeaderValue::from_static(r#"Basic realm="cc-switch", charset="UTF-8""#),
            )
            .body(Body::empty())
            .unwrap_or_else(|_| Response::new(Body::empty()))
    }

    fn forbidden_csrf() -> Response {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::empty())
            .unwrap_or_else(|_| Response::new(Body::empty()))
    }
}

impl tower_http::validate_request::ValidateRequest<Body> for AuthValidator {
    type ResponseBody = Body;

    fn validate(
        &mut self,
        request: &mut Request<Body>,
    ) -> Result<(), Response<Self::ResponseBody>> {
        let Some(auth_header) = request
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        else {
            return Err(Self::unauthorized());
        };

        if !self.is_authorized(auth_header) {
            return Err(Self::unauthorized());
        }

        if let Some(csrf) = &self.csrf_token {
            if request.method() != Method::GET && request.method() != Method::HEAD {
                let token = request
                    .headers()
                    .get("x-csrf-token")
                    .and_then(|v| v.to_str().ok());
                if token != Some(csrf.as_str()) {
                    return Err(Self::forbidden_csrf());
                }
            }
        }

        Ok(())
    }
}

async fn add_hsts_header(
    hsts_enabled: bool,
    req: Request<Body>,
    next: middleware::Next,
) -> Response {
    let mut res = next.run(req).await;
    if hsts_enabled {
        let value = HeaderValue::from_static("max-age=31536000; includeSubDomains");
        res.headers_mut()
            .entry(STRICT_TRANSPORT_SECURITY)
            .or_insert(value);
    }
    res
}

fn token_store_path() -> Option<PathBuf> {
    get_home_dir().map(|home| home.join(".cc-switch").join("web_env"))
}

fn load_or_generate_tokens() -> WebTokens {
    let env_api = env::var("WEB_API_TOKEN").ok();
    let env_csrf = env::var("WEB_CSRF_TOKEN").ok();

    if let Some(api) = env_api {
        let csrf = env_csrf.unwrap_or_else(|| generate_token(16));
        return WebTokens {
            api_token: api,
            csrf_token: csrf,
        };
    }

    if let Some(path) = token_store_path() {
        if let Ok(content) = fs::read_to_string(&path) {
            let mut api = None;
            let mut csrf = None;
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("WEB_API_TOKEN=") {
                    api = Some(val.trim().to_string());
                } else if let Some(val) = line.strip_prefix("WEB_CSRF_TOKEN=") {
                    csrf = Some(val.trim().to_string());
                }
            }
            if let Some(api_val) = api {
                let csrf_val = csrf.unwrap_or_else(|| generate_token(16));
                return WebTokens {
                    api_token: api_val,
                    csrf_token: csrf_val,
                };
            }
        }

        let api = generate_token(48);
        let csrf = env_csrf.unwrap_or_else(|| generate_token(16));
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = atomic_write(
            &path,
            format!("WEB_API_TOKEN={api}\nWEB_CSRF_TOKEN={csrf}\n").as_bytes(),
        );
        log::info!(
            "WEB_API_TOKEN/WEB_CSRF_TOKEN 已生成并写入 {}",
            path.display()
        );
        WebTokens {
            api_token: api,
            csrf_token: csrf,
        }
    } else {
        let api = generate_token(48);
        let csrf = env_csrf.unwrap_or_else(|| generate_token(16));
        WebTokens {
            api_token: api,
            csrf_token: csrf,
        }
    }
}

fn generate_token(len: usize) -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
