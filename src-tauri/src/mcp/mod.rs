/// MCP 服务器管理模块
///
/// 该模块提供 MCP (Model Context Protocol) 服务器的配置管理功能，
/// 支持 Claude、Codex、Gemini 三个应用的统一管理。
///
/// 包含以下子模块：
/// - `core`: 核心配置操作和导入功能
/// - `sync`: 服务器状态同步功能
/// - `conversion`: 配置格式转换
/// - `normalization`: 配置数据规范化
/// - `validation`: 配置验证
// 核心功能模块（从原 mcp.rs 迁移）
mod core;

// 子模块（模块化重构）
pub(crate) mod conversion;
pub(crate) mod normalization;
pub(crate) mod sync;
pub mod validation;

// 从 core 模块导出导入功能
pub use core::{import_from_claude, import_from_codex, import_from_gemini};

// 从 sync 模块导出同步功能
pub use sync::{
    remove_server_from_claude, remove_server_from_codex, remove_server_from_gemini,
    sync_enabled_to_claude, sync_enabled_to_codex, sync_enabled_to_gemini,
    sync_single_server_to_claude, sync_single_server_to_codex, sync_single_server_to_gemini,
};
