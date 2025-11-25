#![cfg(feature = "web-server")]

use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use axum::serve;
use log::{error, info};
use rand::{seq::SliceRandom, thread_rng};
use tokio::{net::TcpListener, signal};

use cc_switch_lib::{
    get_home_dir,
    store::AppState,
    web_api::{create_router, SharedState},
};

fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

fn password_file_path() -> io::Result<PathBuf> {
    let home_dir = get_home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Unable to locate home directory for web password",
        )
    })?;

    Ok(home_dir.join(".cc-switch").join("web_password"))
}

fn generate_password(length: usize) -> String {
    const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const DIGITS: &[u8] = b"0123456789";
    const ALL: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let mut rng = thread_rng();
    let mut chars = Vec::with_capacity(length);

    let mut push_from = |pool: &[u8]| {
        if let Some(ch) = pool.choose(&mut rng) {
            chars.push(*ch as char);
        }
    };

    push_from(LOWER);
    push_from(UPPER);
    push_from(DIGITS);

    while chars.len() < length {
        if let Some(ch) = ALL.choose(&mut rng) {
            chars.push(*ch as char);
        }
    }

    chars.shuffle(&mut rng);
    chars.into_iter().collect()
}

#[cfg(unix)]
fn enforce_permissions(path: &Path) -> io::Result<()> {
    fs::set_permissions(path, PermissionsExt::from_mode(0o600))
}

#[cfg(windows)]
fn enforce_permissions(path: &Path) -> io::Result<()> {
    use std::process::Command;

    let path_str = path.to_string_lossy();

    // Use icacls to remove inheritance and grant only the current user full control.
    let output = Command::new("icacls")
        .args([&*path_str, "/inheritance:r", "/grant:r", "*S-1-3-4:F"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("Failed to set Windows file permissions: {}", stderr);
    }

    Ok(())
}

#[cfg(all(not(unix), not(windows)))]
fn enforce_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn load_or_generate_password() -> Result<(String, PathBuf), Box<dyn std::error::Error>> {
    let path = password_file_path()?;

    if let Ok(existing) = fs::read_to_string(&path) {
        let password = existing.trim().to_string();
        if !password.is_empty() {
            enforce_permissions(&path)?;
            return Ok((password, path));
        }
    }

    let password = generate_password(24);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        options.mode(0o600);
    }

    let mut file = options.open(&path)?;
    file.write_all(password.as_bytes())?;
    enforce_permissions(&path)?;

    Ok((password, path))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    let (password, password_path) = load_or_generate_password()?;

    let state: SharedState = Arc::new(AppState::try_new()?);
    let app = create_router(state, password.clone());

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr: SocketAddr = format!("{host}:{port}").parse().unwrap_or_else(|_| {
        log::warn!("Invalid HOST `{host}`, falling back to 127.0.0.1");
        SocketAddr::from(([127, 0, 0, 1], port))
    });

    let allow_insecure = env::var("ALLOW_HTTP_BASIC_OVER_HTTP")
        .is_ok_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"));
    if addr.ip().is_unspecified() && !allow_insecure {
        log::warn!(
            "当前以 HTTP 监听 0.0.0.0，Basic/Bearer 凭证可能被截获。如需公开，请在反向代理中终止 TLS 并设置 ALLOW_HTTP_BASIC_OVER_HTTP=1"
        );
    } else if !addr.ip().is_loopback() && !allow_insecure {
        log::warn!(
            "监听非本地地址 {}，建议启用 TLS 反代或设置 ALLOW_HTTP_BASIC_OVER_HTTP=1 明确接受风险",
            addr
        );
    }

    info!(
        "Starting web server on http://{} with file-based credentials at {} (username: admin, token stored only on disk)",
        addr,
        password_path.display()
    );
    println!(
        "Web console login -> user: admin, token path: {} (content not printed for safety)",
        password_path.display()
    );

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down cleanly");
    Ok(())
}

async fn shutdown_signal() {
    match signal::ctrl_c().await {
        Ok(()) => info!("Shutdown signal received, terminating server ..."),
        Err(err) => error!("Failed to listen for shutdown signal: {}", err),
    }
}
