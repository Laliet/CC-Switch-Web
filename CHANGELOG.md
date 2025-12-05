# Changelog

All notable changes to CC Switch will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2024-11-30

### Added
- 预编译 server binary：Linux x86_64/aarch64 开箱即用
- Docker 支持：多阶段 Dockerfile 容器化部署
- deploy-web.sh --prebuilt 选项：秒级部署

### Changed  
- 解耦 desktop/web-server feature：web-server 不再依赖 Tauri/GTK/WebKit
- 降低 Rust 版本要求：1.83 → 1.75
- 精简 Web 服务器编译依赖：仅需 libssl-dev, pkg-config

### Fixed
- Web 模式部署不再需要安装桌面 GUI 依赖

## [0.3.0] - 2025-11-29

### ✨ New Features

#### Relay-Pulse 健康检查集成
- **实时健康状态监控**：集成 [Relay-Pulse](https://relaypulse.top) API 提供供应商健康状态监控
  - 自动获取供应商可用性状态（可用/降级/不可用）
  - 显示 24 小时平均可用率百分比
  - 显示 API 响应延迟
- **智能健康数据聚合**：当同一供应商有多个 channel（如 88code 的 vip3/vip5）时，自动聚合为最差状态
  - 状态优先级：unavailable > degraded > available > unknown
  - 可用率取最低值，确保用户了解潜在问题
- **增强的自动故障转移**：基于健康状态而非用量查询脚本进行自动切换
  - 当前供应商不健康时自动切换到健康的备用供应商
  - 切换前检查备用供应商健康状态
- **后端健康检查代理**：解决 CORS 跨域问题
  - 新增 `/api/health/status` 代理端点
  - 后端转发请求到 Relay-Pulse API

### 🔧 Improvements

- **供应商卡片健康指示器**：
  - 彩色圆点显示健康状态（绿色=可用，黄色=降级，红色=不可用，灰色=未知）
  - 圆点旁直接显示可用率百分比（如 `● 95.2%`）
  - 悬停提示显示详细信息（状态、延迟、24小时可用率）
- **Dialog 可访问性改进**：为所有 DialogContent 组件添加 DialogDescription，消除控制台警告

### 📦 Technical Details

- 新增文件：
  - `src/lib/api/healthCheck.ts` - 健康检查 API 模块
  - `src/config/healthCheckMapping.ts` - 供应商名称映射配置
  - `src/hooks/useHealthCheck.ts` - React 健康检查 Hook
  - `src-tauri/src/web_api/handlers/health.rs` - 后端健康检查代理
- 修改文件：
  - `src/components/providers/ProviderCard.tsx` - 添加健康状态显示
  - `src/App.tsx` - 集成健康检查和增强自动故障转移
  - `src-tauri/src/web_api/routes.rs` - 添加健康检查路由

## [0.2.0] - 2025-11-26

### 🎉 版本亮点

本版本为 CC-Switch-Web 项目的首个重大更新，整合了多项安全增强、跨平台兼容性修复和开发者体验改进。

### ✨ New Features

#### Linux 一键启动脚本
- **新增 `scripts/install.sh`**：自动选择架构（x86_64/aarch64）下载 release 资产
- 支持可选 SHA256 校验，保障安装安全
- 可安装到用户目录 (`~/.local/bin`) 或系统目录 (`/usr/local/bin`)
- 自动生成 `.desktop` 文件与应用图标

#### GitHub Actions CI 工作流
- **三平台自动化测试**：Ubuntu、Windows、macOS
- 自动触发 PR 构建与测试
- 支持 `fix/*` 分支测试触发

### 🔒 Security Enhancements

#### JS 沙箱与 Web API 安全增强
- **JavaScript 沙箱隔离**：`rquickjs` 执行环境限制，防止恶意脚本执行
- **API 安全增强**：添加速率限制、请求验证和输入过滤
- **Windows Web 模式安全改进**：修复跨平台安全隐患

#### 原子写入与 unwrap 安全化
- **配置文件原子写入**：防止写入中断导致的配置损坏
- **消除 `unwrap()` 调用**：使用安全的 `?` 操作符和 `match` 模式，避免 panic
- **错误传播改进**：使用 `thiserror` 提供清晰的错误信息

### 🔧 Improvements

#### 跨平台兼容性修复
- **macOS**：修复 Tauri 2.x API 变更导致的编译错误（`window.ns_window()` 返回类型从 `Option` 变为 `Result`）
- **Windows CI**：添加 `dist-web` 占位目录，修复 RustEmbed 在 CI 环境下的编译错误
- **Windows 测试隔离**：新增 `get_home_dir()` 函数，优先检查 `HOME`/`USERPROFILE` 环境变量

#### Rust 版本与依赖调整
- **Axum 0.7 完整迁移**：完成 Web API 框架升级
- **依赖更新**：更新 Cargo.lock 至最新稳定版本
- **Rust edition**：明确指定 2021 edition 和 rust-version 1.83.0

### 🐛 Bug Fixes

#### 测试修复
- 修复 4 个 `app_config` 测试因 `dirs::home_dir()` 在 Windows 上忽略环境变量而失败的问题
- UsageFooter 补充 `backupProviderId` / `onAutoFailover` 入参类型，恢复自动故障切换渲染与类型检查

#### 配置管理修复
- 非 Windows 平台删除 system 环境变量时改为最佳努力移除当前进程变量
- MCP：统一读取旧分应用结构的启用项，切换 Codex 供应商时同步到 `config.toml`

### 📦 Technical Details

- **项目重命名**：更新为 CC-Switch-Web
- **文档更新**：添加 Web 截图和维护说明
- **依赖审计**：确保所有依赖版本安全

### 📝 Statistics

- **总提交数**：11 commits (from v0.1.0 to v0.2.0)
- **主要变更文件**：43 files changed
- **代码行数**：约 +1,500 insertions, -300 deletions

### 🔧 Release Engineering Fixes (by Laliet)

本次发布过程中修复了多个 CI/CD 和签名相关问题：

#### Tauri 签名密钥兼容性
- **scrypt 参数过高**：Minisign 生成的密钥 scrypt 参数超出 Tauri 支持范围，改用 `tauri signer generate --ci --password` 生成兼容密钥
- **GitHub Secret 空格问题**：Actions 变量展开会引入空格（ASCII 32），使用 `env:` 块配合 `tr -d ' \r\n'` 清理空白字符
- **密码环境变量**：`--ci` 标志仍生成加密密钥，需同时配置 `TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

#### CI 工作流修复
- **Cargo target 类型**：`cc-switch-server` 从 `[[bin]]` 移至 `[[example]]` 后，CI 需使用 `--example server` 替代 `--bin cc-switch-server`

#### 公钥格式修复
- **完整 base64 编码**：`tauri.conf.json` 中的 `pubkey` 需包含完整内容（含 `untrusted comment` 行），而非仅第二行

## [0.1.1] - 2025-11-25

### Added
- Linux 一键安装脚本 `scripts/install.sh`：自动选择架构下载 release 资产、可选 SHA256 校验、安装到用户或系统 bin，并生成 `.desktop` 与图标。
- GitHub Actions CI 工作流：支持 Ubuntu、Windows、macOS 三平台自动化测试。

### Fixed
- **跨平台兼容性修复**：
  - macOS：修复 Tauri 2.x API 变更导致的编译错误（`window.ns_window()` 返回类型从 `Option` 变为 `Result`）。
  - Windows CI：添加 `dist-web` 占位目录，修复 RustEmbed 在 CI 环境下的编译错误。
  - Windows 测试隔离：新增 `get_home_dir()` 函数，在 Windows 上优先检查 `HOME`/`USERPROFILE` 环境变量，修复 4 个 `app_config` 测试因 `dirs::home_dir()` 忽略环境变量而失败的问题。
- UsageFooter 补充 `backupProviderId` / `onAutoFailover` 入参类型，恢复自动故障切换渲染与类型检查。
- 非 Windows 删除 system 环境变量时改为最佳努力移除当前进程变量，避免"删除成功"但仍提示冲突的误导。
- MCP：统一读取旧分应用结构的启用项，切换 Codex 供应商时同步到 `config.toml`，修复测试失败。

### Changed
- 版本号更新至 `0.1.1`。

## [0.1.0] - 2025-11-25

### Fixed
- MCP “空配置”首次加载报错：`get_all_servers` 现在在空配置时返回空 Map。
- MCP 兼容接口去除弃用调用：`get_config` 过滤启用应用后返回统一结构。
- 配置导出/导入（Web）：POST `/config/export` 无 body 时返回快照；导入支持直接传完整配置 JSON，修复 415。
- Provider live 同步：返回结构统一为 `{ success, message }`，前端兼容布尔。
- Skill 列表：去重改用唯一 key，避免不同仓库同名目录被折叠。

### Changed
- Web Server：支持 `HOST` 环境变量（默认 `0.0.0.0`）、可选 CORS 环境配置。
- 文档：补充 Web 模式文件选择限制与 CORS 配置说明。
- 版本号更新至 `0.1.0`。

## [3.7.0] - 2025-11-19

### Major Features

#### Gemini CLI Integration

- **Complete Gemini CLI support** - Third major application added alongside Claude Code and Codex
- **Dual-file configuration** - Support for both `.env` and `settings.json` file formats
- **Environment variable detection** - Auto-detect `GOOGLE_GEMINI_BASE_URL`, `GEMINI_MODEL`, etc.
- **MCP management** - Full MCP configuration capabilities for Gemini
- **Provider presets**
  - Google Official (OAuth authentication)
  - PackyCode (partner integration)
  - Custom endpoint support
- **Deep link support** - Import Gemini providers via `ccswitch://` protocol
- **System tray integration** - Quick-switch Gemini providers from tray menu
- **Backend modules** - New `gemini_config.rs` (20KB) and `gemini_mcp.rs`

#### MCP v3.7.0 Unified Architecture

- **Unified management panel** - Single interface for Claude/Codex/Gemini MCP servers
- **SSE transport type** - New Server-Sent Events support alongside stdio/http
- **Smart JSON parser** - Fault-tolerant parsing of various MCP config formats
- **Extended field support** - Preserve custom fields in Codex TOML conversion
- **Codex format correction** - Proper `[mcp_servers]` format (auto-cleanup of incorrect `[mcp.servers]`)
- **Import/export system** - Unified import from Claude/Codex/Gemini live configs
- **UX improvements**
  - Default app selection in forms
  - JSON formatter for config validation
  - Improved layout and visual hierarchy
  - Better validation error messages

#### Claude Skills Management System

- **GitHub repository integration** - Auto-scan and discover skills from GitHub repos
- **Pre-configured repositories**
  - `ComposioHQ/awesome-claude-skills` (curated collection)
  - `anthropics/skills` (official Anthropic skills)
  - `cexll/myclaude` (community, with subdirectory scanning)
- **Lifecycle management**
  - One-click install to `~/.claude/skills/`
  - Safe uninstall with state tracking
  - Update checking (infrastructure ready)
- **Custom repository support** - Add any GitHub repo as a skill source
- **Subdirectory scanning** - Optional `skillsPath` for repos with nested skill directories
- **Backend architecture** - `SkillService` (526 lines) with GitHub API integration
- **Frontend interface**
  - SkillsPage: Browse and manage skills
  - SkillCard: Visual skill presentation
  - RepoManager: Repository management dialog
- **State persistence** - Installation state stored in `skills.json`
- **Full i18n support** - Complete Chinese/English translations (47+ keys)

#### Prompts (System Prompts) Management

- **Multi-preset management** - Create, edit, and switch between multiple system prompts
- **Cross-app support**
  - Claude: `~/.claude/CLAUDE.md`
  - Codex: `~/.codex/AGENTS.md`
  - Gemini: `~/.gemini/GEMINI.md`
- **Markdown editor** - Full-featured CodeMirror 6 editor with syntax highlighting
- **Smart synchronization**
  - Auto-write to live files on enable
  - Content backfill protection (save current before switching)
  - First-launch auto-import from live files
- **Single-active enforcement** - Only one prompt can be active at a time
- **Delete protection** - Cannot delete active prompts
- **Backend service** - `PromptService` (213 lines) with CRUD operations
- **Frontend components**
  - PromptPanel: Main management interface (177 lines)
  - PromptFormModal: Edit dialog with validation (160 lines)
  - MarkdownEditor: CodeMirror integration (159 lines)
  - usePromptActions: Business logic hook (152 lines)
- **Full i18n support** - Complete Chinese/English translations (41+ keys)

#### Deep Link Protocol (ccswitch://)

- **Protocol registration** - `ccswitch://` URL scheme for one-click imports
- **Provider import** - Import provider configurations from URLs or shared links
- **Lifecycle integration** - Deep link handling integrated into app startup
- **Cross-platform support** - Works on Windows, macOS, and Linux

#### Environment Variable Conflict Detection

- **Claude & Codex detection** - Identify conflicting environment variables
- **Gemini auto-detection** - Automatic environment variable discovery
- **Conflict management** - UI for resolving configuration conflicts
- **Prevention system** - Warn before overwriting existing configurations

### New Features

#### Provider Management

- **DouBaoSeed preset** - Added ByteDance's DouBao provider
- **Kimi For Coding** - Moonshot AI coding assistant
- **BaiLing preset** - BaiLing AI integration
- **Removed AnyRouter preset** - Discontinued provider
- **Model configuration** - Support for custom model names in Codex and Gemini
- **Provider notes field** - Add custom notes to providers for better organization

#### Configuration Management

- **Common config migration** - Moved Claude common config snippets from localStorage to `config.json`
- **Unified persistence** - Common config snippets now shared across all apps
- **Auto-import on first launch** - Automatically import configs from live files on first run
- **Backfill priority fix** - Correct priority handling when enabling prompts

#### UI/UX Improvements

- **macOS native design** - Migrated color scheme to macOS native design system
- **Window centering** - Default window position centered on screen
- **Password input fixes** - Disabled Edge/IE reveal and clear buttons
- **URL overflow prevention** - Fixed overflow in provider cards
- **Error notification enhancement** - Copy-to-clipboard for error messages
- **Tray menu sync** - Real-time sync after drag-and-drop sorting

### Improvements

#### Architecture

- **MCP v3.7.0 cleanup** - Removed legacy code and warnings
- **Unified structure** - Default initialization with v3.7.0 unified structure
- **Backward compatibility** - Compilation fixes for older configs
- **Code formatting** - Applied consistent formatting across backend and frontend

#### Platform Compatibility

- **Windows fix** - Resolved winreg API compatibility issue (v0.52)
- **Safe pattern matching** - Replaced `unwrap()` with safe patterns in tray menu

#### Configuration

- **MCP sync on switch** - Sync MCP configs for all apps when switching providers
- **Gemini form sync** - Fixed form fields syncing with environment editor
- **Gemini config reading** - Read from both `.env` and `settings.json`
- **Validation improvements** - Enhanced input validation and boundary checks

#### Internationalization

- **JSON syntax fixes** - Resolved syntax errors in locale files
- **App name i18n** - Added internationalization support for app names
- **Deduplicated labels** - Reused providerForm keys to reduce duplication
- **Gemini MCP title** - Added missing Gemini MCP panel title

### Bug Fixes

#### Critical Fixes

- **Usage script validation** - Added input validation and boundary checks
- **Gemini validation** - Relaxed validation when adding providers
- **TOML quote normalization** - Handle CJK quotes to prevent parsing errors
- **MCP field preservation** - Preserve custom fields in Codex TOML editor
- **Password input** - Fixed white screen crash (FormLabel → Label)

#### Stability

- **Tray menu safety** - Replaced unwrap with safe pattern matching
- **Error isolation** - Tray menu update failures don't block main operations
- **Import classification** - Set category to custom for imported default configs

#### UI Fixes

- **Model placeholders** - Removed misleading model input placeholders
- **Base URL population** - Auto-fill base URL for non-official providers
- **Drag sort sync** - Fixed tray menu order after drag-and-drop

### Technical Improvements

#### Code Quality

- **Type safety** - Complete TypeScript type coverage across codebase
- **Test improvements** - Simplified boolean assertions in tests
- **Clippy warnings** - Fixed `uninlined_format_args` warnings
- **Code refactoring** - Extracted templates, optimized logic flows

#### Dependencies

- **Tauri** - Updated to 2.8.x series
- **Rust dependencies** - Added `anyhow`, `zip`, `serde_yaml`, `tempfile` for Skills
- **Frontend dependencies** - Added CodeMirror 6 packages for Markdown editor
- **winreg** - Updated to v0.52 (Windows compatibility)

#### Performance

- **Startup optimization** - Removed legacy migration scanning
- **Lock management** - Improved RwLock usage to prevent deadlocks
- **Background query** - Enabled background mode for usage polling

### Statistics

- **Total commits**: 85 commits from v3.6.0 to v3.7.0
- **Code changes**: 152 files changed, 18,104 insertions(+), 3,732 deletions(-)
- **New modules**:
  - Skills: 2,034 lines (21 files)
  - Prompts: 1,302 lines (20 files)
  - Gemini: ~1,000 lines (multiple files)
  - MCP refactor: ~3,000 lines (refactored)

### Strategic Positioning

v3.7.0 represents a major evolution from "Provider Switcher" to **"All-in-One AI CLI Management Platform"**:

1. **Capability Extension** - Skills provide external ability integration
2. **Behavior Customization** - Prompts enable AI personality presets
3. **Configuration Unification** - MCP v3.7.0 eliminates app silos
4. **Ecosystem Openness** - Deep links enable community sharing
5. **Multi-AI Support** - Claude/Codex/Gemini trinity
6. **Intelligent Detection** - Auto-discovery of environment conflicts

### Notes

- Users upgrading from v3.1.0 or earlier should first upgrade to v3.2.x for one-time migration
- Skills and Prompts management are new features requiring no migration
- Gemini CLI support requires Gemini CLI to be installed separately
- MCP v3.7.0 unified structure is backward compatible with previous configs

## [3.6.0] - 2025-11-07

### ✨ New Features

- **Provider Duplicate** - Quick duplicate existing provider configurations for easy variant creation
- **Edit Mode Toggle** - Show/hide drag handles to optimize editing experience
- **Custom Endpoint Management** - Support multi-endpoint configuration for aggregator providers
- **Usage Query Enhancements**
  - Auto-refresh interval: Support periodic automatic usage query
  - Test Script API: Validate JavaScript scripts before execution
  - Template system expansion: Custom blank template, support for access token and user ID parameters
- **Configuration Editor Improvements**
  - Add JSON format button
  - Real-time TOML syntax validation for Codex configuration
- **Auto-sync on Directory Change** - When switching Claude/Codex config directories (e.g., WSL environment), automatically sync current provider to new directory without manual operation
- **Load Live Config When Editing Active Provider** - When editing the currently active provider, prioritize displaying the actual effective configuration to protect user manual modifications
- **New Provider Presets** - DMXAPI, Azure Codex, AnyRouter, AiHubMix, MiniMax
- **Partner Promotion Mechanism** - Support ecosystem partner promotion (e.g., Zhipu GLM Z.ai)

### 🔧 Improvements

- **Configuration Directory Switching**
  - Introduced unified post-change sync utility (`postChangeSync.ts`)
  - Auto-sync current providers to new directory when changing Claude/Codex config directories
  - Perfect support for WSL environment switching
  - Auto-sync after config import to ensure immediate effectiveness
  - Use Result pattern for graceful error handling without blocking main flow
  - Distinguish "fully successful" and "partially successful" states for precise user feedback
- **UI/UX Enhancements**
  - Provider cards: Unique icons and color identification
  - Unified border design system across all components
  - Drag interaction optimization: Push effect animation, improved handle icons
  - Enhanced current provider visual feedback
  - Dialog size standardization and layout consistency
  - Form experience: Optimized model placeholders, simplified provider hints, category-specific hints
- **Complete Internationalization Coverage**
  - Error messages internationalization
  - Tray menu internationalization
  - All UI components internationalization
- **Usage Display Moved Inline** - Usage display moved next to enable button

### 🐛 Bug Fixes

- **Configuration Sync**
  - Fixed `apiKeyUrl` priority issue
  - Fixed MCP sync-to-other-side functionality failure
  - Fixed sync issues after config import
  - Prevent silent fallback and data loss on config error
- **Usage Query**
  - Fixed auto-query interval timing issue
  - Ensure refresh button shows loading animation on click
- **UI Issues**
  - Fixed name collision error (`get_init_error` command)
  - Fixed language setting rollback after successful save
  - Fixed language switch state reset (dependency cycle)
  - Fixed edit mode button alignment
- **Configuration Management**
  - Fixed Codex API Key auto-sync
  - Fixed endpoint speed test functionality
  - Fixed provider duplicate insertion position (next to original provider)
  - Fixed custom endpoint preservation in edit mode
- **Startup Issues**
  - Force exit on config error (no silent fallback)
  - Eliminate code duplication causing initialization errors

### 🏗️ Technical Improvements (For Developers)

**Backend Refactoring (Rust)** - Completed 5-phase refactoring:

- **Phase 1**: Unified error handling (`AppError` + i18n error messages)
- **Phase 2**: Command layer split by domain (`commands/{provider,mcp,config,settings,plugin,misc}.rs`)
- **Phase 3**: Integration tests and transaction mechanism (config snapshot + failure rollback)
- **Phase 4**: Extracted Service layer (`services/{provider,mcp,config,speedtest}.rs`)
- **Phase 5**: Concurrency optimization (`RwLock` instead of `Mutex`, scoped guard to avoid deadlock)

**Frontend Refactoring (React + TypeScript)** - Completed 4-stage refactoring:

- **Stage 1**: Test infrastructure (vitest + MSW + @testing-library/react)
- **Stage 2**: Extracted custom hooks (`useProviderActions`, `useMcpActions`, `useSettings`, `useImportExport`, etc.)
- **Stage 3**: Component splitting and business logic extraction
- **Stage 4**: Code cleanup and formatting unification

**Testing System**:

- Hooks unit tests 100% coverage
- Integration tests covering key processes (App, SettingsDialog, MCP Panel)
- MSW mocking backend API to ensure test independence

**Code Quality**:

- Unified parameter format: All Tauri commands migrated to camelCase (Tauri 2 specification)
- `AppType` renamed to `AppId`: Semantically clearer
- Unified parsing with `FromStr` trait: Centralized `app` parameter parsing
- Eliminate code duplication: DRY violations cleanup
- Remove unused code: `missing_param` helper function, deprecated `tauri-api.ts`, redundant `KimiModelSelector` component

**Internal Optimizations**:

- **Removed Legacy Migration Logic**: v3.6 removed v1 config auto-migration and copy file scanning logic
  - ✅ **Impact**: Improved startup performance, cleaner code
  - ✅ **Compatibility**: v2 format configs fully compatible, no action required
  - ⚠️ **Note**: Users upgrading from v3.1.0 or earlier should first upgrade to v3.2.x or v3.5.x for one-time migration, then upgrade to v3.6
- **Command Parameter Standardization**: Backend unified to use `app` parameter (values: `claude` or `codex`)
  - ✅ **Impact**: More standardized code, friendlier error prompts
  - ✅ **Compatibility**: Frontend fully adapted, users don't need to care about this change

### 📦 Dependencies

- Updated to Tauri 2.8.x
- Updated to TailwindCSS 4.x
- Updated to TanStack Query v5.90.x
- Maintained React 18.2.x and TypeScript 5.3.x

## [3.5.0] - 2025-01-15

### ⚠ Breaking Changes

- Tauri 命令仅接受参数 `app`（取值：`claude`/`codex`）；移除对 `app_type`/`appType` 的兼容。
- 前端类型命名统一为 `AppId`（移除 `AppType` 导出），变量命名统一为 `appId`。

### ✨ New Features

- **MCP (Model Context Protocol) Management** - Complete MCP server configuration management system
  - Add, edit, delete, and toggle MCP servers in `~/.claude.json`
  - Support for stdio and http server types with command validation
  - Built-in templates for popular MCP servers (mcp-fetch, etc.)
  - Real-time enable/disable toggle for MCP servers
  - Atomic file writing to prevent configuration corruption
- **Configuration Import/Export** - Backup and restore your provider configurations
  - Export all configurations to JSON file with one click
  - Import configurations with validation and automatic backup
  - Automatic backup rotation (keeps 10 most recent backups)
  - Progress modal with detailed status feedback
- **Endpoint Speed Testing** - Test API endpoint response times
  - Measure latency to different provider endpoints
  - Visual indicators for connection quality
  - Help users choose the fastest provider

### 🔧 Improvements

- Complete internationalization (i18n) coverage for all UI components
- Enhanced error handling and user feedback throughout the application
- Improved configuration file management with better validation
- Added new provider presets: Longcat, kat-coder
- Updated GLM provider configurations with latest models
- Refined UI/UX with better spacing, icons, and visual feedback
- Enhanced tray menu functionality and responsiveness
- **Standardized release artifact naming** - All platform releases now use consistent version-tagged filenames:
  - macOS: `CC-Switch-v{version}-macOS.tar.gz` / `.zip`
  - Windows: `CC-Switch-v{version}-Windows.msi` / `-Portable.zip`
  - Linux: `CC-Switch-v{version}-Linux.AppImage` / `.deb`

### 🐛 Bug Fixes

- Fixed layout shifts during provider switching
- Improved config file path handling across different platforms
- Better error messages for configuration validation failures
- Fixed various edge cases in configuration import/export

### 📦 Technical Details

- Enhanced `import_export.rs` module with backup management
- New `claude_mcp.rs` module for MCP configuration handling
- Improved state management and lock handling in Rust backend
- Better TypeScript type safety across the codebase

## [3.4.0] - 2025-10-01

### ✨ Features

- Enable internationalization via i18next with a Chinese default and English fallback, plus an in-app language switcher
- Add Claude plugin sync while retiring the legacy VS Code integration controls (Codex no longer requires settings.json edits)
- Extend provider presets with optional API key URLs and updated models, including DeepSeek-V3.1-Terminus and Qwen3-Max
- Support portable mode launches and enforce a single running instance to avoid conflicts

### 🔧 Improvements

- Allow minimizing the window to the system tray and add macOS Dock visibility management for tray workflows
- Refresh the Settings modal with a scrollable layout, save icon, and cleaner language section
- Smooth provider toggle states with consistent button widths/icons and prevent layout shifts when switching between Claude and Codex
- Adjust the Windows MSI installer to target per-user LocalAppData and improve component tracking reliability

### 🐛 Fixes

- Remove the unnecessary OpenAI auth requirement from third-party provider configurations
- Fix layout shifts while switching app types with Claude plugin sync enabled
- Align Enable/In Use button states to avoid visual jank across app views

## [3.3.0] - 2025-09-22

### ✨ Features

- Add “Apply to VS Code / Remove from VS Code” actions on provider cards, writing settings for Code/Insiders/VSCodium variants _(Removed in 3.4.x)_
- Enable VS Code auto-sync by default with window broadcast and tray hooks so Codex switches sync silently _(Removed in 3.4.x)_
- Extend the Codex provider wizard with display name, dedicated API key URL, and clearer guidance
- Introduce shared common config snippets with JSON/TOML reuse, validation, and consistent error surfaces

### 🔧 Improvements

- Keep the tray menu responsive when the window is hidden and standardize button styling and copy
- Disable modal backdrop blur on Linux (WebKitGTK/Wayland) to avoid freezes; restore the window when clicking the macOS Dock icon
- Support overriding config directories on WSL, refine placeholders/descriptions, and fix VS Code button wrapping on Windows
- Add a `created_at` timestamp to provider records for future sorting and analytics

### 🐛 Fixes

- Correct regex escapes and common snippet trimming in the Codex wizard to prevent validation issues
- Harden the VS Code sync flow with more reliable TOML/JSON parsing while reducing layout jank
- Bundle `@codemirror/lint` to reinstate live linting in config editors

## [3.2.0] - 2025-09-13

### ✨ New Features

- System tray provider switching with dynamic menu for Claude/Codex
- Frontend receives `provider-switched` events and refreshes active app
- Built-in update flow via Tauri Updater plugin with dismissible UpdateBadge

### 🔧 Improvements

- Single source of truth for provider configs; no duplicate copy files
- One-time migration imports existing copies into `config.json` and archives originals
- Duplicate provider de-duplication by name + API key at startup
- Atomic writes for Codex `auth.json` + `config.toml` with rollback on failure
- Logging standardized (Rust): use `log::{info,warn,error}` instead of stdout prints
- Tailwind v4 integration and refined dark mode handling

### 🐛 Fixes

- Remove/minimize debug console logs in production builds
- Fix CSS minifier warnings for scrollbar pseudo-elements
- Prettier formatting across codebase for consistent style

### 📦 Dependencies

- Tauri: 2.8.x (core, updater, process, opener, log plugins)
- React: 18.2.x · TypeScript: 5.3.x · Vite: 5.x

### 🔄 Notes

- `connect-src` CSP remains permissive for compatibility; can be tightened later as needed

## [3.1.1] - 2025-09-03

### 🐛 Bug Fixes

- Fixed the default codex config.toml to match the latest modifications
- Improved provider configuration UX with custom option

### 📝 Documentation

- Updated README with latest information

## [3.1.0] - 2025-09-01

### ✨ New Features

- **Added Codex application support** - Now supports both Claude Code and Codex configuration management
  - Manage auth.json and config.toml for Codex
  - Support for backup and restore operations
  - Preset providers for Codex (Official, PackyCode)
  - API Key auto-write to auth.json when using presets
- **New UI components**
  - App switcher with segmented control design
  - Dual editor form for Codex configuration
  - Pills-style app switcher with consistent button widths
- **Enhanced configuration management**
  - Multi-app config v2 structure (claude/codex)
  - Automatic v1→v2 migration with backup
  - OPENAI_API_KEY validation for non-official presets
  - TOML syntax validation for config.toml

### 🔧 Technical Improvements

- Unified Tauri command API with app_type parameter
- Backward compatibility for app/appType parameters
- Added get_config_status/open_config_folder/open_external commands
- Improved error handling for empty config.toml

### 🐛 Bug Fixes

- Fixed config path reporting and folder opening for Codex
- Corrected default import behavior when main config is missing
- Fixed non_snake_case warnings in commands.rs

## [3.0.0] - 2025-08-27

### 🚀 Major Changes

- **Complete migration from Electron to Tauri 2.0** - The application has been completely rewritten using Tauri, resulting in:
  - **90% reduction in bundle size** (from ~150MB to ~15MB)
  - **Significantly improved startup performance**
  - **Native system integration** without Chromium overhead
  - **Enhanced security** with Rust backend

### ✨ New Features

- **Native window controls** with transparent title bar on macOS
- **Improved file system operations** using Rust for better performance
- **Enhanced security model** with explicit permission declarations
- **Better platform detection** using Tauri's native APIs

### 🔧 Technical Improvements

- Migrated from Electron IPC to Tauri command system
- Replaced Node.js file operations with Rust implementations
- Implemented proper CSP (Content Security Policy) for enhanced security
- Added TypeScript strict mode for better type safety
- Integrated Rust cargo fmt and clippy for code quality

### 🐛 Bug Fixes

- Fixed bundle identifier conflict on macOS (changed from .app to .desktop)
- Resolved platform detection issues
- Improved error handling in configuration management

### 📦 Dependencies

- **Tauri**: 2.8.2
- **React**: 18.2.0
- **TypeScript**: 5.3.0
- **Vite**: 5.0.0

### 🔄 Migration Notes

For users upgrading from v2.x (Electron version):

- Configuration files remain compatible - no action required
- The app will automatically migrate your existing provider configurations
- Window position and size preferences have been reset to defaults

#### Backup on v1→v2 Migration (cc-switch internal config)

- When the app detects an old v1 config structure at `~/.cc-switch/config.json`, it now creates a timestamped backup before writing the new v2 structure.
- Backup location: `~/.cc-switch/config.v1.backup.<timestamp>.json`
- This only concerns cc-switch's own metadata file; your actual provider files under `~/.claude/` and `~/.codex/` are untouched.

### 🛠️ Development

- Added `pnpm typecheck` command for TypeScript validation
- Added `pnpm format` and `pnpm format:check` for code formatting
- Rust code now uses cargo fmt for consistent formatting

## [2.0.0] - Previous Electron Release

### Features

- Multi-provider configuration management
- Quick provider switching
- Import/export configurations
- Preset provider templates

---

## [1.0.0] - Initial Release

### Features

- Basic provider management
- Claude Code integration
- Configuration file handling

## [Unreleased]

### ⚠️ Breaking Changes

- **Runtime auto-migration from v1 to v2 config format has been removed**
  - `MultiAppConfig::load()` no longer automatically migrates v1 configs
  - When a v1 config is detected, the app now returns a clear error with migration instructions
  - **Migration path**: Install v3.2.x to perform one-time auto-migration, OR manually edit `~/.cc-switch/config.json` to v2 format
  - **Rationale**: Separates concerns (load() should be read-only), fail-fast principle, simplifies maintenance
  - Related: `app_config.rs` (v1 detection improved with structural analysis), `app_config_load.rs` (comprehensive test coverage added)

- **Legacy v1 copy file migration logic has been removed**
  - Removed entire `migration.rs` module (435 lines) that handled one-time migration from v3.1.0 to v3.2.0
  - No longer scans/merges legacy copy files (`settings-*.json`, `auth-*.json`, `config-*.toml`)
  - No longer archives copy files or performs automatic deduplication
  - **Migration path**: Users upgrading from v3.1.0 must first upgrade to v3.2.x to automatically migrate their configurations
  - **Benefits**: Improved startup performance (no file scanning), reduced code complexity, cleaner codebase

- **Tauri commands now only accept `app` parameter**
  - Removed legacy `app_type`/`appType` compatibility paths
  - Explicit error with available values when unknown `app` is provided

### 🔧 Improvements

- Unified `AppType` parsing: centralized to `FromStr` implementation, command layer no longer implements separate `parse_app()`, reducing code duplication and drift
- Localized and user-friendly error messages: returns bilingual (Chinese/English) hints for unsupported `app` values with a list of available options
- Simplified startup logic: Only ensures config structure exists, no migration overhead

### 🧪 Tests

- Added unit tests covering `AppType::from_str`: case sensitivity, whitespace trimming, unknown value error messages
- Added comprehensive config loading tests:
  - `load_v1_config_returns_error_and_does_not_write`
  - `load_v1_with_extra_version_still_treated_as_v1`
  - `load_invalid_json_returns_parse_error_and_does_not_write`
  - `load_valid_v2_config_succeeds`
