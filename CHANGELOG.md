# Changelog

All notable changes to CC Switch will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-01-11

### Migration Notes / 迁移说明

- CORS is same-origin by default.
- `CORS_ALLOW_ORIGINS="*"` is ignored.
- `ALLOW_LAN_CORS=1` or `CC_SWITCH_LAN_CORS=1` auto-allows private LAN origins.
- Binding to `0.0.0.0` still requires `ALLOW_LAN_CORS=1`.

## [0.7.1] - 2026-01-10

### Fixes / 修复

- CI 修复/类型检查修复

## [0.7.0] - 2026-01-09

### Features / 新特性

- Skills UI status line shows cache hit/background refresh
- Web API base override with safer validation in `WebLoginDialog`
- Web mode reads live settings into default provider without switching current
- Web switch syncs to live config and returns explicit errors on failure

### Performance / 性能

- Skills repository cache with ETag/Last-Modified conditional refresh, cache TTL via `CC_SWITCH_SKILLS_CACHE_TTL_SECS`, and fallback to cache on fetch failure

## [0.6.0] - 2025-12-26

### 🔒 Security / 安全修复

**Critical / 严重 (5个)：**
- **修复 Config 路径遍历漏洞** - `services/config.rs`: `/config/export` 和 `/config/import` 接受用户控制的 `filePath`，可导致任意文件读写。添加路径消毒、规范化和白名单校验
- **修复 Skills 路径遍历漏洞** - `services/skill.rs`, `web_api/handlers/skills.rs`: `directory` 参数未规范化，`../` 可删除任意目录。添加中央验证器，拒绝 `..`、空值和绝对路径
- **修复 XSS 漏洞** - `lib/api/adapter.ts:649`: `open_external` 可打开 `javascript:`/`data:` URL 执行脚本。添加 URL scheme 验证，只允许 http/https
- **修复 env_manager 路径遍历** - `services/env_manager.rs`: 备份/恢复路径可被用户控制修改任意文件。添加规范化路径验证，限制到备份目录和已知 shell 配置文件
- **修复非幂等操作重试** - `lib/api/adapter.ts:677`: 重试循环适用于突变操作可能双重应用副作用。限制重试只对 GET/HEAD 请求

**High / 高优先级 (7个)：**
- **修复资源泄漏** - `services/skill.rs`: 临时目录在错误路径/超时时未清理，泄漏 `/tmp`。改用 RAII 临时目录，自动 drop 清理
- **修复阻塞异步** - `services/skill.rs:842`: CPU 密集的 zip 解压阻塞 tokio 运行时。使用 `spawn_blocking` 移到阻塞线程池
- **修复备份 ID 冲突** - `services/config.rs:21`: 秒级时间戳可能覆盖同秒内的备份。改用毫秒时间戳+单调计数器
- **修复导入竞态条件** - `services/config.rs:111,123`: 导入路径未与 AppState 同步，并发操作可能数据丢失。添加 AppState 写锁下解析再应用
- **修复静默数据丢失** - `services/config.rs:191`: 非字符串 `config` 被视为 `None`，写空文件。添加类型验证
- **修复 app_config 竞态** - `app_config.rs:343,375`: 读-修改-写无文件锁。添加 `config.json.lock` 文件锁
- **修复配置文件权限** - `config.rs:107,293,312`: 配置文件和备份写入时未硬化权限，可能泄露 API 密钥。添加敏感路径检测和私有权限强制（Unix 0600）

### 🐛 Bug Fixes / Bug 修复

**MCP 组件修复 (8个)：**
- **修复 McpFormModal 错误残留** - `:236` TOML 验证通过后未清除 `configError`；`:196,218,305` 设置 `formId` 后从未重置 `idError`
- **修复 McpFormModal 内存泄漏** - `:412` 异步操作 `finally` 中 setState 可能作用于已卸载组件。添加 `isMounted` 守卫
- **修复 McpFormModal 无效类型** - `:385,395` 无效 `type` 值可能被保存。添加显式类型验证
- **修复 wizard 不必要解析** - `:143` wizard 关闭时仍解析 TOML/JSON。延迟到 `isWizardOpen` 时
- **修复 UnifiedMcpPanel null** - `:67` 假设 `server.apps` 总存在，旧配置崩溃。添加 optional chaining
- **修复 UnifiedMcpPanel 卸载** - `:91` 异步 setState 在卸载后。添加 `isMountedRef` 守卫
- **修复 McpListItem null** - `:98` 同上，`server.apps` 可能 undefined。默认 `false`
- **修复 MCP 类型验证** - `validation.rs:13` 非字符串 `type` 默认为 stdio 允许恶意配置；`useMcpValidation.ts:61` JSON 解析两次。单次解析+严格类型校验

**MCP 后端修复 (4个)：**
- **修复 MCP 转换验证** - `conversion.rs:99,142,174`: `json_server_to_toml_table` 从不验证 spec。转换前调用 `validate_server_spec`
- **修复 MCP 同步验证** - `sync.rs:37,58,112,132`: 同步路径不验证必需字段，无效 spec 传播到活动配置。同步前验证，跳过无效条目

**Skills 组件修复 (3个)：**
- **修复 SkillsPage 竞态** - `:158` `loadSkills` 无条件更新，重叠调用覆盖新数据导致 UI 过期。添加请求 ID 门控
- **修复 SkillsPage 错误边界** - `:32` 无错误边界，渲染错误会导致整个页面崩溃。添加本地 `ErrorBoundary`
- **修复 SkillCard 卸载** - `:38,47` `setLoading(false)` 在 `finally` 中，卸载后可能 setState。添加 `isMounted` 守卫

**API 和网络修复 (5个)：**
- **修复 healthCheck 超时** - `:142` GUI/Tauri 路径无超时，后端挂起导致 promise 永不 resolve；`:147` 超时只覆盖初始 fetch 不覆盖响应体读取。添加 `withTimeout` helper
- **修复 adapter 错误解析** - `:685` 非 OK 响应总是读取为文本，丢失结构化错误信息。解析 JSON 错误 payload
- **修复 adapter 空字符串** - `:558` 空字符串 `content` 被丢弃，web 模式空配置导入失败。严格字符串检查
- **修复 adapter 参数验证** - `:279` `update_providers_sort_order` 不验证参数，undefined 变成 `{}`。添加 `requireArg`
- **修复 web_api 404** - `mod.rs:211` 根 `/*path` 捕获所有，未知 API 路径返回 SPA HTML 而非 404。添加 API fallback handler

**CORS 修复 (2个)：**
- **修复 CORS 配置** - `mod.rs:145,196` `CORS_ALLOW_ORIGINS="*"` 被忽略但中间件仍启用，CORS 失败。修复逻辑
- **修复 CORS HEAD 方法** - `mod.rs:135` `allow_methods` 遗漏 HEAD。添加 HEAD

**错误处理修复 (5个)：**
- **修复 prompt.rs 错误忽略** - `:80` `read_to_string` 错误静默忽略；`:81` 仅空白内容被视为空；`:106` `trim()` 去重忽略空白变更。返回读取错误，比较原始内容
- **修复 prompt.rs panic** - `:97,112,175` `duration_since(UNIX_EPOCH).unwrap()` 系统时间异常时 panic。添加 `unix_timestamp` helper
- **修复 app_config panic** - `:514` 同上。处理 `Err` 并 fallback 到 0
- **修复 skillErrorParser** - `:19` code-only 错误回退到原始文本；`:20` 解析的 JSON 未验证类型。验证 JSON 形状，规范化 context
- **修复 adapter 类型安全** - `:5` `CommandArgs` 是 `Record<string, any>`；`:624` web invoke 返回 null 转为 T。使用 `unknown` 类型，添加 null 处理

**配置预设修复 (5个)：**
- **修复 DMXAPI apiKeyField** - `claudeProviderPresets.ts:286` 使用 `ANTHROPIC_API_KEY` 但未设 `apiKeyField`
- **修复 AiHubMix/DMXAPI endpoints** - `:278,292` `endpointCandidates` 搞反了
- **修复 healthCheckMapping** - `:108` `aihubmix.com` 映射到 `dmxapi`，与专用预设冲突。添加 AiHubMix 映射
- **修复 Codex 配置清空不同步** - `providerConfigUtils.ts:511`, `useCodexConfigState.ts:162,216`, `useBaseUrlState.ts:112`: Base URL/Model 清空后被回写成旧值。修复写回逻辑和预设切换时的 reset 逻辑
- **修复 Gemini OAuth 切换** - `services/provider.rs:1636`, `gemini_config.rs:355`: 切换到通用 API Key 供应商时 `settings.json` 仍保留 `oauth-personal`。添加显式 auth type 写入

### ♿ Accessibility / 无障碍

- **修复 PromptListItem 无障碍** - `:36` Prompt toggle 无无障碍名称；`:54,63` 图标按钮依赖 `title`。添加 `aria-label`
- **修复 PromptToggle** - 接受 `label` prop 以支持无障碍
- **修复 WebLoginDialog 无障碍** - `WebLoginDialog.tsx:117-127`: 密码表单缺少用户名字段导致 Chrome 警告。添加隐藏的 username 字段支持密码管理器
- **修复 DialogContent 无障碍** - `App.tsx:603`: Skills 对话框缺少 `DialogDescription` 导致 Radix UI 警告。添加 screen-reader-only 描述
- **新增 favicon** - `src/public/favicon.ico`, `src/index.html:7`: 添加网站图标解决 404

### ⚡ Performance / 性能优化

- **优化 RepoManager** - `:42` `getSkillCount` 每次渲染 O(repos*skills)。使用 `useMemo` 预计算 skill counts
- **优化 ProviderForm** - `:675,738` `shouldShowApiKey` 每次渲染/按键触发 JSON 解析。memoize API key 可见性
- **优化 useTemplateValues** - `:126` `collectTemplatePaths` 每次变更遍历完整配置。缓存 template 路径

### 🧪 Tests / 测试

- 新增 `services/skill.rs` 路径验证测试
- Rust 单元测试：49 个全部通过
- 前端单元测试：58 个全部通过
- TypeScript 类型检查通过

### 📁 Changed Files / 变更文件

**Rust 后端 (src-tauri/src/):**
- `app_config.rs` - 文件锁、时间戳安全处理
- `services/config.rs` - 路径遍历防护、备份 ID、竞态修复、权限硬化
- `services/skill.rs` - 路径验证、RAII 临时目录、spawn_blocking
- `services/prompt.rs` - 错误处理、时间戳安全
- `services/env_manager.rs` - 路径规范化验证
- `services/provider.rs` - Gemini OAuth 切换修复
- `gemini_config.rs` - API Key 写入助手
- `web_api/mod.rs` - API 404 处理、CORS 修复
- `web_api/handlers/config.rs` - 路径消毒
- `web_api/handlers/skills.rs` - 目录验证
- `mcp/validation.rs` - 类型验证
- `mcp/conversion.rs` - spec 验证
- `mcp/sync.rs` - 同步前验证
- `commands/import_export.rs` - 导入流程修复

**前端 (src/):**
- `lib/api/adapter.ts` - XSS 防护、重试逻辑、类型安全
- `lib/api/healthCheck.ts` - 超时处理
- `lib/errors/skillErrorParser.ts` - 错误解析验证
- `components/mcp/McpFormModal.tsx` - 状态管理、卸载守卫
- `components/mcp/UnifiedMcpPanel.tsx` - null 检查、卸载守卫
- `components/mcp/McpListItem.tsx` - null 检查
- `components/mcp/useMcpValidation.ts` - 类型验证
- `components/skills/SkillsPage.tsx` - 竞态处理、错误边界
- `components/skills/SkillCard.tsx` - 卸载守卫
- `components/skills/RepoManager.tsx` - 性能优化
- `components/prompts/PromptListItem.tsx` - 无障碍
- `components/prompts/PromptToggle.tsx` - 无障碍
- `components/providers/forms/ProviderForm.tsx` - 性能优化
- `components/providers/forms/hooks/useTemplateValues.ts` - 性能优化
- `components/providers/forms/hooks/useCodexConfigState.ts` - Codex 配置清空修复
- `components/providers/forms/hooks/useBaseUrlState.ts` - Base URL 清空修复
- `components/WebLoginDialog.tsx` - 密码表单无障碍
- `utils/providerConfigUtils.ts` - 配置写回逻辑修复
- `config/claudeProviderPresets.ts` - 配置修复
- `config/healthCheckMapping.ts` - 映射修复
- `App.tsx` - DialogContent 无障碍
- `public/favicon.ico` - 新增网站图标
- `index.html` - favicon 引用
- `i18n/locales/en.json` - 新增验证消息
- `i18n/locales/zh.json` - 新增验证消息

**配置:**
- `vite.config.mts` - publicDir 配置
- `vite.config.web.mts` - publicDir 配置

**测试:**
- `tests/msw/state.ts` - 新增 MCP/环境冲突 mock
- `tests/msw/handlers.ts` - 新增统一 MCP handler

---

## [0.5.4] - 2025-12-17

### 🐛 Bug Fixes / Bug 修复

**Critical / 严重：**
- **彻底修复 `crypto.randomUUID` 在非安全上下文不可用** - 新增 `src/utils/uuid.ts`，实现三级降级策略：
  1. 优先使用 `crypto.randomUUID()`（安全上下文：HTTPS / localhost）
  2. 降级到 `crypto.getRandomValues()` + RFC 4122 v4 格式化（非安全上下文但有 Crypto API）
  3. 最终降级到 `Math.random()` 模板（极老浏览器/特殊环境）
- **修复添加供应商失败** - `mutations.ts`: 替换直接调用 `crypto.randomUUID()` 为安全的 `generateUUID()`

### 🧪 Tests / 测试

- 新增 `tests/utils/uuid.test.ts` - UUID 生成器完整测试
  - 格式验证（8-4-4-4-12 hex，v4 版本位 + variant 位）
  - 唯一性测试（1000 个 UUID 无重复）
  - `crypto.randomUUID` 不可用时降级测试
  - `crypto.getRandomValues` 不可用时降级测试
- 测试数量：142 → 146（+4）

### 📁 Changed Files / 变更文件

- `src/utils/uuid.ts` (新增) - 安全的 UUID 生成器，三级降级策略
- `src/lib/query/mutations.ts` - 导入并使用 `generateUUID()`
- `tests/utils/uuid.test.ts` (新增) - UUID 生成器单元测试

---

## [0.5.3] - 2025-12-16

### 🔒 Security / 安全修复

**Critical / 严重：**
- **修复 API Key 日志泄露** - `DeepLinkImportDialog.tsx`: 添加 `maskApiKey()` 函数，日志和 UI 展示均脱敏（仅保留前后各 2-4 位）
- **修复 XSS 漏洞** - `ApiKeySection.tsx`: 添加 `isSafeUrl()` 校验，仅允许 http/https 协议链接，阻止 `javascript:` 等危险 scheme

**High / 高优先级：**
- **修复 URL schema 验证不足** - `provider.ts`: 添加 `isHttpOrHttpsUrl()` refine 校验，拒绝 `javascript:`/`data:` 等危险协议

### 🐛 Bug Fixes / Bug 修复

**Web 模式修复：**
- **修复 405 错误** - `adapter.ts`: 移除 `/api/tauri/*` fallback，未知命令抛出明确错误；`read_live_provider_settings` 返回 null
- **修复健康检查 401** - `healthCheck.ts`: Web 模式下自动添加 Authorization 头
- **修复导出配置 401** - `useImportExport.ts`: Web 模式导出配置时添加 Authorization 头
- **修复登录校验逻辑** - `App.tsx`: `return true` → `return response.ok`，只有 2xx 状态才视为成功

**竞态条件与内存泄漏：**
- **修复 useEffect 竞态条件** - `App.tsx`: 添加 `cancelled` 标记，cleanup 时正确取消订阅，避免事件监听泄漏
- **修复闭包陷阱** - `usePromptActions.ts`: 深拷贝快照 + 函数式更新 + 写入令牌机制，防止并发触发时数据覆盖

**Promise rejection 处理：**
- **修复未处理 Promise rejection** - `App.tsx`: `handleAutoFailover` 顶层包 try/catch
- **修复未处理 Promise rejection** - `UsageFooter.tsx`: 改为 `void onAutoFailover?.(...)`

**其他修复：**
- **修复生产环境日志污染** - `useHealthCheck.ts`: 仅 `import.meta.env.DEV` 下输出轮询日志
- **修复 localStorage 异常** - `useSettingsForm.ts`: 添加 try/catch，Safari 隐私模式下优雅降级
- **修复 checkUpdate 抛错** - `UpdateContext.tsx`: 不再 throw，改为写入 error 状态
- **修复闭包依赖遗漏** - `SettingsDialog.tsx`: 补齐 `closeAfterSave` 依赖

### 🧪 Tests / 测试

- 新增 `tests/lib/adapter.auth.test.ts` - 未知命令错误测试
- 新增 `tests/lib/providerSchema.test.ts` - URL schema 校验测试
- 新增 `tests/components/ApiKeySection.test.tsx` - XSS 防护测试
- 测试数量：139 → 142（+3）

### 📁 Changed Files / 变更文件

- `src/App.tsx` - 竞态条件 + rejection 处理
- `src/components/DeepLinkImportDialog.tsx` - API Key 脱敏
- `src/components/UsageFooter.tsx` - void 处理
- `src/components/providers/forms/shared/ApiKeySection.tsx` - XSS 防护
- `src/components/settings/AboutSection.tsx` - checkUpdate 返回值适配
- `src/components/settings/SettingsDialog.tsx` - 依赖项修复
- `src/contexts/UpdateContext.tsx` - 不再 throw
- `src/hooks/useHealthCheck.ts` - DEV-only 日志
- `src/hooks/useImportExport.ts` - 认证头修复
- `src/hooks/usePromptActions.ts` - 闭包陷阱修复
- `src/hooks/useSettingsForm.ts` - localStorage try/catch
- `src/lib/api/adapter.ts` - 405 错误修复
- `src/lib/api/healthCheck.ts` - 认证头修复
- `src/lib/schemas/provider.ts` - URL schema 校验

### v0.5.2 (2025-12-16)

#### 🐛 Bug Fixes
- 修复 Web 模式下 `crypto.randomUUID` 在非安全上下文（HTTP）中不可用的问题
- 修复 Web 模式下 `process.env` 在浏览器中不可用导致的错误
- 修复 Web 开发模式下登录认证流程（Basic Auth + CSRF Token）
- 修复 Skills API 因远程仓库获取超时导致的 AbortError
- 修复 ComposioHQ/awesome-claude-skills 仓库分支名配置（main → master）

#### ⚡ Improvements
- Skills API 现在返回警告信息，远程仓库获取失败时仍显示本地技能
- 增加 Skills 仓库下载超时时间（HTTP: 120s，总超时: 180s）
- 增加前端 API 请求超时时间（30s → 180s）
- 添加 Web 登录对话框，支持手动输入密码认证
- 添加 CSRF Token API 端点 `GET /api/system/csrf-token`

## [0.5.1] - 2025-12-14

### 🔒 Security / 安全修复

**高优先级：**
- **修复 Web 服务器认证绕过漏洞** - 移除 API Token 注入，强制使用 Basic Auth
  - 之前：apiToken 被注入到 HTML 中，任何访问者都能获得完整 API 权限
  - 之后：只注入 csrfToken（防伪用），API 访问必须通过 Basic Auth 输入密码
- **修复 CSRF Token 注入 XSS 风险** - 使用 serde_json 序列化并转义特殊字符
- **修复 CSRF Token 文件权限** - 显式设置 `~/.cc-switch/web_env` 为 0600

**安全增强：**
- 添加安全响应头：X-Frame-Options、X-Content-Type-Options、Referrer-Policy
- CORS 配置添加 X-CSRF-Token 到允许的 headers
- 移除 Bearer Token 认证方式，仅保留 Basic Auth

### 🧪 Tests / 测试
- 新增后端 Web 认证测试 (`src-tauri/tests/web_auth.rs`)
- 新增前端认证相关测试 (`tests/lib/adapter.auth.test.ts`)

### 📖 Documentation / 文档
- README.md/README_ZH.md 添加详细的 Web 服务器安全说明
- 添加环境变量配置表格

### 📁 Changed Files / 变更文件
- `src-tauri/src/web_api/mod.rs`
- `src/lib/api/adapter.ts`
- `src/components/UsageFooter.tsx`
- `src-tauri/tests/web_auth.rs` (new)
- `tests/lib/adapter.auth.test.ts` (new)
- `README.md`
- `README_ZH.md`

## [0.5.0] - 2025-12-11

### 🐛 Bug Fixes / Bug 修复

**高优先级修复：**
- **修复 switchProvider 错误处理** - `useProviderActions.ts`：切换供应商失败时错误不再被吞掉，现在会正确抛出让调用方处理
- **修复 mutateAsync 未处理 rejection** - `App.tsx`：添加 try/catch 处理编辑、删除、复制供应商操作的异步错误
- **修复全局可变状态竞态** - `providerConfigUtils.ts`：`updateTomlCommonConfigSnippet` 改为纯函数，消除 `previousCommonSnippet` 全局状态泄漏
- **修复 useImportExport 闭包陷阱** - `useImportExport.ts`：依赖数组添加 `selectedFileContent`，修复导入文件时使用旧内容的问题

**中优先级修复：**
- **修复健康检查可用率误导** - `healthCheck.ts`：`mergeHealth` 无数据时不再默认 100% 可用，改为 `undefined`
- **修复 localStorage 崩溃** - `UpdateContext.tsx`：Safari 隐私模式等环境下 localStorage 访问添加保护，优雅降级
- **修复 MarkdownEditor/JsonEditor 闭包陷阱** - 使用 `useRef` 存储 `onChange` 回调，避免编辑器重建
- **修复 PromptPanel 状态泄漏** - `PromptPanel.tsx`：关闭面板时重置 confirmDialog 状态
- **修复导入定时器未清理** - `useImportExport.ts`：多次导入时清理旧定时器，避免跨次运行竞态

**健壮性改进：**
- **添加 baseUrl 验证** - `codexProviderPresets.ts`：生成第三方配置时验证和转义 URL，防止无效 TOML
- **添加 fetch 超时/重试** - `adapter.ts`：Web 模式添加 30s 超时和重试机制，避免请求挂起
- **添加健康检查超时** - `healthCheck.ts`：添加 10s AbortController 超时

### 📁 Changed Files / 变更文件
- `src/hooks/useProviderActions.ts`
- `src/hooks/useImportExport.ts`
- `src/utils/providerConfigUtils.ts`
- `src/contexts/UpdateContext.tsx`
- `src/lib/api/healthCheck.ts`
- `src/lib/api/adapter.ts`
- `src/components/MarkdownEditor.tsx`
- `src/components/JsonEditor.tsx`
- `src/components/prompts/PromptPanel.tsx`
- `src/config/codexProviderPresets.ts`
- `src/App.tsx`
- `tests/hooks/useProviderActions.test.tsx`

## [0.4.4] - 2025-12-07

### 🐛 Bug Fixes / Bug 修复
- Fix Windows test failure in `app_config` tests / 修复 Windows 上 app_config 测试失败
  - Reset app_store override and settings cache when TempHome changes / TempHome 变更时重置缓存路径

## [0.4.3] - 2025-12-06

### 🐛 Bug Fixes / Bug 修复
- **Fix blank window on macOS Sequoia (15.x)** / **修复 macOS Sequoia (15.x) 上应用窗口空白的问题**
  - Enable `withGlobalTauri` to inject `__TAURI__` global on macOS WebKit / 启用 `withGlobalTauri` 以在 macOS WebKit 上注入 `__TAURI__` 全局对象
  - Expand `assetProtocol.scope` from `[]` to `["**"]` to allow resource loading / 将 `assetProtocol.scope` 从 `[]` 扩展为 `["**"]` 以允许资源加载
  - Update CSP to include `asset:` and `tauri:` schemes / 更新 CSP 以包含 `asset:` 和 `tauri:` 协议
  - Improve Tauri runtime detection to check both `__TAURI__` and `__TAURI_INTERNALS__` / 改进 Tauri 运行时检测，同时检查 `__TAURI__` 和 `__TAURI_INTERNALS__`

## [0.4.2] - 2025-12-06

### 🔒 Security / 安全修复
- Fix Windows `atomic_write` command injection vulnerability (config.rs) / 修复 Windows atomic_write 命令注入漏洞 (config.rs)
- Fix ZIP path traversal vulnerability (skill.rs) / 修复 ZIP 路径遍历攻击漏洞 (skill.rs)

### 🐛 Bug Fixes / Bug 修复
- Fix Web UI not showing installed MCPs by auto-importing external configs (services/mcp.rs) / 修复 Web 版本无法显示已安装 MCP 的问题 - 添加自动导入外部配置功能 (services/mcp.rs)
- Fix `import_from_codex` exiting early on unknown types (mcp.rs) / 修复 import_from_codex 遇到未知类型时提前退出的问题 (mcp.rs)
- Fix MCP management panel showing empty lists on query failures (UnifiedMcpPanel.tsx) / 修复 MCP 管理面板查询失败时显示空列表的问题 (UnifiedMcpPanel.tsx)

### 🖥️ Cross-Platform / 跨平台兼容
- Handle PATHEXT/.exe when validating Windows commands (claude_mcp.rs) / 修复 Windows 命令验证缺少 PATHEXT/.exe 处理的问题 (claude_mcp.rs)
- Normalize `skills_path` separators on Windows (skill.rs) / 修复 skills_path 路径分隔符在 Windows 上的问题 (skill.rs)

### ✨ Enhancements / 功能增强
- Add debounce and loading states to the MCP management panel to prevent repeated clicks / MCP 管理面板添加操作防抖和 loading 状态，防止重复点击
- Add `useSkills` React Query hooks / 新增 useSkills React Query hooks

### 🧪 Tests / 测试
- Add MCP validation and TOML conversion unit tests (mcp.rs) / 新增 MCP 验证和 TOML 转换单元测试 (mcp.rs)
- Add skills path parsing and metadata parsing unit tests (skill.rs) / 新增 Skills 路径解析和元数据解析单元测试 (skill.rs)
- Add `useSkills` hooks frontend tests / 新增 useSkills hooks 前端测试
- Update test docs with a full bilingual guide (tests/README.md) / 更新测试文档 (tests/README.md) - 完整的中英双语测试指南

### 📦 CI/CD
- Add GitHub Actions frontend test job / GitHub Actions CI 新增前端测试 job

## [0.4.1] - 2025-12-05

### Fixed
- 修复 GitHub 用户名变更导致的下载链接失效问题（LITLAY2004 → Laliet）
- 修复 Docker 镜像名大小写问题（ghcr.io 要求全小写）
- 修复 Dockerfile 中 Rust 版本过旧导致 Cargo.lock v4 解析失败（1.75 → 1.83）

### Changed
- 更新所有文档和脚本中的 GitHub 仓库链接
- Docker 镜像地址更新为 `ghcr.io/laliet/cc-switch-web`（注意小写）

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
