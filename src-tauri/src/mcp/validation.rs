//! MCP 服务器配置验证模块

use serde_json::Value;
use crate::error::AppError;

/// 基础校验：允许 stdio/http/sse；或省略 type（视为 stdio）。对应必填字段存在
pub(crate) fn validate_server_spec(spec: &Value) -> Result<(), AppError> {
    if !spec.is_object() {
        return Err(AppError::McpValidation(
            "MCP 服务器连接定义必须为 JSON 对象".into(),
        ));
    }
    let type_value = spec.get("type");
    if let Some(val) = type_value {
        if !val.is_string() {
            return Err(AppError::McpValidation(
                "MCP 服务器 type 必须为字符串".into(),
            ));
        }
    }
    let t_opt = type_value.and_then(|x| x.as_str());
    // 支持三种：stdio/http/sse；若缺省 type 则按 stdio 处理（与社区常见 .mcp.json 一致）
    let is_stdio = matches!(t_opt, Some("stdio")) || type_value.is_none();
    let is_http = matches!(t_opt, Some("http"));
    let is_sse = matches!(t_opt, Some("sse"));

    if !(is_stdio || is_http || is_sse) {
        return Err(AppError::McpValidation(
            "MCP 服务器 type 必须是 'stdio'、'http' 或 'sse'（或省略表示 stdio）".into(),
        ));
    }

    if is_stdio {
        let cmd = spec.get("command").and_then(|x| x.as_str()).unwrap_or("");
        if cmd.trim().is_empty() {
            return Err(AppError::McpValidation(
                "stdio 类型的 MCP 服务器缺少 command 字段".into(),
            ));
        }
    }
    if is_http {
        let url = spec.get("url").and_then(|x| x.as_str()).unwrap_or("");
        if url.trim().is_empty() {
            return Err(AppError::McpValidation(
                "http 类型的 MCP 服务器缺少 url 字段".into(),
            ));
        }
    }
    if is_sse {
        let url = spec.get("url").and_then(|x| x.as_str()).unwrap_or("");
        if url.trim().is_empty() {
            return Err(AppError::McpValidation(
                "sse 类型的 MCP 服务器缺少 url 字段".into(),
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)] // v3.7.0: 旧的验证逻辑，保留用于未来可能的迁移
pub(crate) fn validate_mcp_entry(entry: &Value) -> Result<(), AppError> {
    let obj = entry
        .as_object()
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目必须为 JSON 对象".into()))?;

    let server = obj
        .get("server")
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目缺少 server 字段".into()))?;
    validate_server_spec(server)?;

    for key in ["name", "description", "homepage", "docs"] {
        if let Some(val) = obj.get(key) {
            if !val.is_string() {
                return Err(AppError::McpValidation(format!(
                    "MCP 服务器 {key} 必须为字符串"
                )));
            }
        }
    }

    if let Some(tags) = obj.get("tags") {
        let arr = tags
            .as_array()
            .ok_or_else(|| AppError::McpValidation("MCP 服务器 tags 必须为字符串数组".into()))?;
        if !arr.iter().all(|item| item.is_string()) {
            return Err(AppError::McpValidation(
                "MCP 服务器 tags 必须为字符串数组".into(),
            ));
        }
    }

    if let Some(enabled) = obj.get("enabled") {
        if !enabled.is_boolean() {
            return Err(AppError::McpValidation(
                "MCP 服务器 enabled 必须为布尔值".into(),
            ));
        }
    }

    Ok(())
}
