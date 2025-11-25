use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use tauri_plugin_store::StoreExt;

use crate::{
    config::{atomic_write, get_home_dir},
    error::AppError,
};

/// Store 中的键名
const STORE_KEY_APP_CONFIG_DIR: &str = "app_config_dir_override";

/// 缓存当前的 app_config_dir 覆盖路径，避免存储 AppHandle
static APP_CONFIG_DIR_OVERRIDE: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();

fn override_cache() -> &'static RwLock<Option<PathBuf>> {
    APP_CONFIG_DIR_OVERRIDE.get_or_init(|| RwLock::new(None))
}

fn update_cached_override(value: Option<PathBuf>) {
    if let Ok(mut guard) = override_cache().write() {
        *guard = value;
    }
}

/// 获取缓存中的 app_config_dir 覆盖路径
pub fn get_app_config_dir_override() -> Option<PathBuf> {
    if let Ok(guard) = override_cache().read() {
        if guard.is_some() {
            return guard.clone();
        }
    }

    // 对于 Web Server 模式，允许直接从磁盘读取默认的 app_paths.json
    if let Some(from_disk) = read_override_from_disk() {
        update_cached_override(Some(from_disk.clone()));
        return Some(from_disk);
    }

    None
}

fn read_override_from_store(app: &tauri::AppHandle) -> Option<PathBuf> {
    let store = match app.store_builder("app_paths.json").build() {
        Ok(store) => store,
        Err(e) => {
            log::warn!("无法创建 Store: {e}");
            return None;
        }
    };

    match store.get(STORE_KEY_APP_CONFIG_DIR) {
        Some(Value::String(path_str)) => {
            let path_str = path_str.trim();
            if path_str.is_empty() {
                return None;
            }

            let path = resolve_path(path_str);

            if !path.exists() {
                log::warn!(
                    "Store 中配置的 app_config_dir 不存在: {path:?}\n\
                     将使用默认路径。"
                );
                return None;
            }

            log::info!("使用 Store 中的 app_config_dir: {path:?}");
            Some(path)
        }
        Some(_) => {
            log::warn!("Store 中的 {STORE_KEY_APP_CONFIG_DIR} 类型不正确，应为字符串");
            None
        }
        None => None,
    }
}

/// 从 Store 刷新 app_config_dir 覆盖值并更新缓存
pub fn refresh_app_config_dir_override(app: &tauri::AppHandle) -> Option<PathBuf> {
    let value = read_override_from_store(app);
    update_cached_override(value.clone());
    value
}

/// 写入 app_config_dir 到 Tauri Store
pub fn set_app_config_dir_to_store(
    app: &tauri::AppHandle,
    path: Option<&str>,
) -> Result<(), AppError> {
    let store = app
        .store_builder("app_paths.json")
        .build()
        .map_err(|e| AppError::Message(format!("创建 Store 失败: {e}")))?;

    match path {
        Some(p) => {
            let trimmed = p.trim();
            if !trimmed.is_empty() {
                store.set(STORE_KEY_APP_CONFIG_DIR, Value::String(trimmed.to_string()));
                log::info!("已将 app_config_dir 写入 Store: {trimmed}");
            } else {
                store.delete(STORE_KEY_APP_CONFIG_DIR);
                log::info!("已从 Store 中删除 app_config_dir 配置");
            }
        }
        None => {
            store.delete(STORE_KEY_APP_CONFIG_DIR);
            log::info!("已从 Store 中删除 app_config_dir 配置");
        }
    }

    store
        .save()
        .map_err(|e| AppError::Message(format!("保存 Store 失败: {e}")))?;

    refresh_app_config_dir_override(app);
    Ok(())
}

/// 解析路径，支持 ~ 开头的相对路径
fn resolve_path(raw: &str) -> PathBuf {
    if raw == "~" {
        if let Some(home) = get_home_dir() {
            return home;
        }
    } else if let Some(stripped) = raw.strip_prefix("~/") {
        if let Some(home) = get_home_dir() {
            return home.join(stripped);
        }
    } else if let Some(stripped) = raw.strip_prefix("~\\") {
        if let Some(home) = get_home_dir() {
            return home.join(stripped);
        }
    }

    PathBuf::from(raw)
}

/// 从旧的 settings.json 迁移 app_config_dir 到 Store
pub fn migrate_app_config_dir_from_settings(app: &tauri::AppHandle) -> Result<(), AppError> {
    // app_config_dir 已从 settings.json 移除，此函数保留但不再执行迁移
    // 如果用户在旧版本设置过 app_config_dir，需要在 Store 中手动配置
    log::info!("app_config_dir 迁移功能已移除，请在设置中重新配置");

    let _ = refresh_app_config_dir_override(app);
    Ok(())
}

fn store_path() -> Option<PathBuf> {
    let home = get_home_dir()?;
    Some(home.join(".cc-switch").join("app_paths.json"))
}

fn read_override_from_disk() -> Option<PathBuf> {
    let path = store_path()?;
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(&path).ok()?;
    let value: Value = serde_json::from_str(&content).ok()?;
    match value {
        Value::Object(map) => map
            .get(STORE_KEY_APP_CONFIG_DIR)
            .and_then(|v| v.as_str())
            .map(resolve_path),
        _ => None,
    }
}

/// 在无 Tauri 环境下（如 Web Server）设置 app_config_dir 覆盖路径并写入磁盘。
#[allow(dead_code)]
pub fn set_app_config_dir_override_standalone(path: Option<&str>) -> Result<(), AppError> {
    let store_path = store_path()
        .ok_or_else(|| AppError::Message("无法获取用户主目录以写入 app_paths.json".to_string()))?;

    let mut data: Value = if store_path.exists() {
        fs::read_to_string(&store_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()))
    } else {
        Value::Object(serde_json::Map::new())
    };

    let map = data
        .as_object_mut()
        .ok_or_else(|| AppError::Message("app_paths.json 内容格式无效".to_string()))?;

    match path {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                map.remove(STORE_KEY_APP_CONFIG_DIR);
                update_cached_override(None);
            } else {
                map.insert(
                    STORE_KEY_APP_CONFIG_DIR.to_string(),
                    Value::String(trimmed.to_string()),
                );
                update_cached_override(Some(resolve_path(trimmed)));
            }
        }
        None => {
            map.remove(STORE_KEY_APP_CONFIG_DIR);
            update_cached_override(None);
        }
    }

    if let Some(parent) = store_path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }

    let serialized = serde_json::to_string_pretty(&data)
        .map_err(|e| AppError::Message(format!("序列化 app_paths.json 失败: {e}")))?;
    atomic_write(&store_path, serialized.as_bytes())?;

    Ok(())
}
