# cc-switch-web — Claude Code / Codex / Gemini 的 Web/无头助手

English | 中文 | [更新日志](CHANGELOG.md)

cc-switch-web 是 **cc-switch** 桌面版的二次开发，面向 Web/服务器和无头场景。保留统一的供应商/MCP/技能/提示词管理，增强 Web 安全默认，重点支持云端部署。

## 为什么选择 cc-switch-web（对比桌面版）
- 云端/无头友好：纯 Web Server，无需桌面环境。
- 统一 HTTP 接口：Claude Code / Codex / Gemini 一套 API。
- 模板更丰富：新增 MCP/技能/供应商预设。
- 默认更安全：自动生成 Basic Auth 密码；默认同源，跨域需显式开启。
- 可控监听：`HOST`/`PORT` 可配，便于 HTTPS 反代。
- 智能容灾：备用供应商自动切换，转发异常自动兜底。

## 亮点
- Claude/Codex/Gemini 供应商切换与实时写入。
- 统一 MCP 管理，支持跨客户端导入/导出。
- 技能市场：仓库扫描 + 一键安装。
- 提示词管理：内置 CodeMirror 编辑器。
- 导入导出 + 备份轮换，目录重定向（适配 WSL/云同步）。

## 快速开始（Web）
```bash
pnpm install
pnpm build:web
cd src-tauri
cargo build --release --features web-server --bin cc-switch-server
HOST=0.0.0.0 PORT=3000 ./target/release/cc-switch-server
```
- 登录：`admin` / `~/.cc-switch/web_password` 自动生成的密码。
- CORS：默认同源；跨域需设置 `CORS_ALLOW_ORIGINS`（可选 `CORS_ALLOW_CREDENTIALS=true`）。
- Web 模式不支持系统文件/目录选择器，请手动输入路径。

## 截图
| Skills 市场 | 提示词编辑 | 高级设置 |
| :--: | :--: | :--: |
| ![Skills](assets/screenshots/web-skills.png) | ![Prompt](assets/screenshots/web-prompt.png) | ![Settings](assets/screenshots/web-settings.png) |

*（请将截图放在上述路径下）*

## 关于上游项目
本项目基于 Jason Young（farion1231）的开源 **cc-switch**。上游通过 Tauri 桌面端统一了供应商切换、MCP 管理、技能和提示词，并具备完善的本地化与安全特性。cc-switch-web 在此基础上增加 Web/服务器运行模式、CORS 控制、默认 Basic Auth、更多模板，以及云端/无头部署的文档支持。感谢上游作者和贡献者的基础工作。

## 维护说明
新上线的 Web/无头版本，部分细节仍在打磨。如果你遇到 bug 或有功能建议，欢迎提 issue。看到反馈后一周内会集中更新，之后每周定期维护，目标是把它打造成云端放心使用的可靠工具。
## 项目结构（关键）
- `src/` React + TypeScript 前端
- `src-tauri/` Rust 后端（Tauri/Web Server）
- `src-tauri/src/web_api/` Axum HTTP API（Web 模式）
- `dist-web/` Web 构建产物（不提交）
- `tests/` Bash + MSW + Vitest 测试

## 使用命令
- 开发（桌面渲染器）：`pnpm dev:renderer`
- 构建 Web 资源：`pnpm build:web`
- 运行 Web Server：`HOST=0.0.0.0 PORT=3000 ./src-tauri/target/release/cc-switch-server`
- 构建 Server：`cd src-tauri && cargo build --release --features web-server --bin cc-switch-server`
- Web API 测试：`bash tests/run-all.sh`（需要运行中的 server，依赖 curl/jq）

## 技术栈
- 前端：React 18、TypeScript、Vite、Tailwind、TanStack Query、Radix UI、CodeMirror
- 后端：Rust、Axum、Tauri（共享逻辑）、tower-http
- 工具：pnpm、Vitest、MSW、Bash + curl/jq（API 测试）

## 测试覆盖
- `tests/api`、`tests/integration`：Bash API/集成测试（供应商、设置、MCP、用量、持久化）。
- MSW/Vitest 组件 & Hook 测试。
- `bash tests/run-all.sh` 可覆盖 Web API（需运行服务器）。

## 更新日志
参见 [CHANGELOG.md](CHANGELOG.md)（当前版本：v0.1.0）。

## 项目亮点回顾
- 原生 Web/Server 支持，默认安全（Basic Auth + 同源）。
+- 统一管理 Claude/Codex/Gemini 的供应商/MCP/技能/提示词。
+- 丰富预设与备用供应商自动切换。
+- 导入/导出含备份，支持 WSL/云同步目录重定向。
