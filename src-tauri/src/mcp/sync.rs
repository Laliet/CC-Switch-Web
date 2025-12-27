use serde_json::Value;
use std::collections::HashMap;

use crate::app_config::{McpConfig, MultiAppConfig};
use crate::config::write_text_file;
use crate::error::AppError;

use super::conversion::json_server_to_toml_table;
use super::validation::validate_server_spec;

// ============================================================================
// 辅助函数（从 mcp.rs 迁移）
// ============================================================================

/// 返回已启用的 MCP 服务器（过滤 enabled==true）
fn collect_enabled_servers(cfg: &McpConfig) -> HashMap<String, Value> {
    let mut out = HashMap::new();
    for (id, entry) in cfg.servers.iter() {
        let enabled = entry
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !enabled {
            continue;
        }
        match extract_server_spec(entry) {
            Ok(spec) => {
                out.insert(id.clone(), spec);
            }
            Err(err) => {
                log::warn!("跳过无效的 MCP 条目 '{id}': {err}");
            }
        }
    }
    out
}

fn extract_server_spec(entry: &Value) -> Result<Value, AppError> {
    let obj = entry
        .as_object()
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目必须为 JSON 对象".into()))?;
    let server = obj
        .get("server")
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目缺少 server 字段".into()))?;

    if !server.is_object() {
        return Err(AppError::McpValidation(
            "MCP 服务器 server 字段必须为 JSON 对象".into(),
        ));
    }

    validate_server_spec(server)?;

    Ok(server.clone())
}

// ============================================================================
// 全量同步函数
// ============================================================================

/// 将 config.json 中 enabled==true 的项投影写入 ~/.claude.json
pub fn sync_enabled_to_claude(config: &MultiAppConfig) -> Result<(), AppError> {
    let enabled = collect_enabled_servers(&config.mcp.claude);
    crate::claude_mcp::set_mcp_servers_map(&enabled)
}

/// 将 config.json 中 Codex 的 enabled==true 项以 TOML 形式写入 ~/.codex/config.toml
///
/// 格式策略：
/// - 唯一正确格式：[mcp_servers] 顶层表（Codex 官方标准）
/// - 自动清理错误格式：[mcp.servers]（如果存在）
/// - 读取现有 config.toml；若语法无效则报错，不尝试覆盖
/// - 仅更新 `mcp_servers` 表，保留其它键
/// - 仅写入启用项；无启用项时清理 mcp_servers 表
pub fn sync_enabled_to_codex(config: &MultiAppConfig) -> Result<(), AppError> {
    use toml_edit::{Item, Table};

    // 1) 收集启用项（Codex 维度）
    let enabled = collect_enabled_servers(&config.mcp.codex);

    // 2) 读取现有 config.toml 文本；保持无效 TOML 的错误返回（不覆盖文件）
    let base_text = crate::codex_config::read_and_validate_codex_config_text()?;

    // 3) 使用 toml_edit 解析（允许空文件）
    let mut doc = if base_text.trim().is_empty() {
        toml_edit::DocumentMut::default()
    } else {
        base_text
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| AppError::McpValidation(format!("解析 config.toml 失败: {e}")))?
    };

    // 4) 清理可能存在的错误格式 [mcp.servers]
    if let Some(mcp_item) = doc.get_mut("mcp") {
        if let Some(tbl) = mcp_item.as_table_like_mut() {
            if tbl.contains_key("servers") {
                log::warn!("检测到错误的 MCP 格式 [mcp.servers]，正在清理并迁移到 [mcp_servers]");
                tbl.remove("servers");
            }
        }
    }

    // 5) 构造目标 servers 表（稳定的键顺序）
    if enabled.is_empty() {
        // 无启用项：移除 mcp_servers 表
        doc.as_table_mut().remove("mcp_servers");
    } else {
        // 构建 servers 表
        let mut servers_tbl = Table::new();
        let mut ids: Vec<_> = enabled.keys().cloned().collect();
        ids.sort();
        for id in ids {
            let spec = enabled.get(&id).expect("spec must exist");
            // 复用通用转换函数（已包含扩展字段支持）
            match json_server_to_toml_table(spec) {
                Ok(table) => {
                    servers_tbl[&id[..]] = Item::Table(table);
                }
                Err(err) => {
                    log::error!("跳过无效的 MCP 服务器 '{id}': {err}");
                }
            }
        }
        // 使用唯一正确的格式：[mcp_servers]
        doc["mcp_servers"] = Item::Table(servers_tbl);
    }

    // 6) 写回（仅改 TOML，不触碰 auth.json）；toml_edit 会尽量保留未改区域的注释/空白/顺序
    let new_text = doc.to_string();
    let path = crate::codex_config::get_codex_config_path();
    write_text_file(&path, &new_text)?;
    Ok(())
}

/// 将 config.json 中 enabled==true 的项投影写入 ~/.gemini/settings.json
pub fn sync_enabled_to_gemini(config: &MultiAppConfig) -> Result<(), AppError> {
    let enabled = collect_enabled_servers(&config.mcp.gemini);
    crate::gemini_mcp::set_mcp_servers_map(&enabled)
}

// ============================================================================
// 单项同步函数
// ============================================================================

/// 将单个 MCP 服务器同步到 Claude live 配置
pub fn sync_single_server_to_claude(
    _config: &MultiAppConfig,
    id: &str,
    server_spec: &Value,
) -> Result<(), AppError> {
    validate_server_spec(server_spec)?;

    // 读取现有的 MCP 配置
    let current = crate::claude_mcp::read_mcp_servers_map()?;

    // 创建新的 HashMap，包含现有的所有服务器 + 当前要同步的服务器
    let mut updated = current;
    updated.insert(id.to_string(), server_spec.clone());

    // 写回
    crate::claude_mcp::set_mcp_servers_map(&updated)
}

/// 将单个 MCP 服务器同步到 Codex live 配置
/// 始终使用 Codex 官方格式 [mcp_servers]，并清理可能存在的错误格式 [mcp.servers]
pub fn sync_single_server_to_codex(
    _config: &MultiAppConfig,
    id: &str,
    server_spec: &Value,
) -> Result<(), AppError> {
    use toml_edit::Item;

    validate_server_spec(server_spec)?;

    // 读取现有的 config.toml
    let config_path = crate::codex_config::get_codex_config_path();

    let mut doc = if config_path.exists() {
        let content =
            std::fs::read_to_string(&config_path).map_err(|e| AppError::io(&config_path, e))?;
        content
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| AppError::McpValidation(format!("解析 Codex config.toml 失败: {e}")))?
    } else {
        toml_edit::DocumentMut::new()
    };

    // 清理可能存在的错误格式 [mcp.servers]
    if let Some(mcp_item) = doc.get_mut("mcp") {
        if let Some(tbl) = mcp_item.as_table_like_mut() {
            if tbl.contains_key("servers") {
                log::warn!("检测到错误的 MCP 格式 [mcp.servers]，正在清理并迁移到 [mcp_servers]");
                tbl.remove("servers");
            }
        }
    }

    // 确保 [mcp_servers] 表存在
    if !doc.contains_key("mcp_servers") {
        doc["mcp_servers"] = toml_edit::table();
    }

    // 将 JSON 服务器规范转换为 TOML 表
    let toml_table = json_server_to_toml_table(server_spec)?;

    // 使用唯一正确的格式：[mcp_servers]
    doc["mcp_servers"][id] = Item::Table(toml_table);

    // 写回文件
    write_text_file(&config_path, &doc.to_string())?;

    Ok(())
}

/// 将单个 MCP 服务器同步到 Gemini live 配置
pub fn sync_single_server_to_gemini(
    _config: &MultiAppConfig,
    id: &str,
    server_spec: &Value,
) -> Result<(), AppError> {
    validate_server_spec(server_spec)?;

    // 读取现有的 MCP 配置
    let current = crate::gemini_mcp::read_mcp_servers_map()?;

    // 创建新的 HashMap，包含现有的所有服务器 + 当前要同步的服务器
    let mut updated = current;
    updated.insert(id.to_string(), server_spec.clone());

    // 写回
    crate::gemini_mcp::set_mcp_servers_map(&updated)
}

// ============================================================================
// 删除函数
// ============================================================================

/// 从 Claude live 配置中移除单个 MCP 服务器
pub fn remove_server_from_claude(id: &str) -> Result<(), AppError> {
    // 读取现有的 MCP 配置
    let mut current = crate::claude_mcp::read_mcp_servers_map()?;

    // 移除指定服务器
    current.remove(id);

    // 写回
    crate::claude_mcp::set_mcp_servers_map(&current)
}

/// 从 Codex live 配置中移除单个 MCP 服务器
/// 从正确的 [mcp_servers] 表中删除，同时清理可能存在于错误位置 [mcp.servers] 的数据
pub fn remove_server_from_codex(id: &str) -> Result<(), AppError> {
    let config_path = crate::codex_config::get_codex_config_path();

    if !config_path.exists() {
        return Ok(()); // 文件不存在，无需删除
    }

    let content =
        std::fs::read_to_string(&config_path).map_err(|e| AppError::io(&config_path, e))?;

    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| AppError::McpValidation(format!("解析 Codex config.toml 失败: {e}")))?;

    // 从正确的位置删除：[mcp_servers]
    if let Some(mcp_servers) = doc.get_mut("mcp_servers").and_then(|s| s.as_table_mut()) {
        mcp_servers.remove(id);
    }

    // 同时清理可能存在于错误位置的数据：[mcp.servers]（如果存在）
    if let Some(mcp_table) = doc.get_mut("mcp").and_then(|t| t.as_table_mut()) {
        if let Some(servers) = mcp_table.get_mut("servers").and_then(|s| s.as_table_mut()) {
            if servers.remove(id).is_some() {
                log::warn!("从错误的 MCP 格式 [mcp.servers] 中清理了服务器 '{id}'");
            }
        }
    }

    // 写回文件
    write_text_file(&config_path, &doc.to_string())?;

    Ok(())
}

/// 从 Gemini live 配置中移除单个 MCP 服务器
pub fn remove_server_from_gemini(id: &str) -> Result<(), AppError> {
    // 读取现有的 MCP 配置
    let mut current = crate::gemini_mcp::read_mcp_servers_map()?;

    // 移除指定服务器
    current.remove(id);

    // 写回
    crate::gemini_mcp::set_mcp_servers_map(&current)
}
