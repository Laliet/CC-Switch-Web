# cc-switch-web — Web/Headless Assistant for Claude Code, Codex & Gemini CLI

English | [中文](README_ZH.md) | [Changelog](CHANGELOG.md)

cc-switch-web is a second-development fork of the original **cc-switch** desktop app, rebuilt for web/server and headless environments. It keeps unified provider/MCP/skills/prompt management, adds safer web defaults, and focuses on cloud compatibility.

## Why cc-switch-web (vs desktop)
- Cloud/headless ready: run as a web server, no GUI needed.
- Unified HTTP API for Claude Code, Codex, Gemini.
- Rich presets: extra MCP/skill/provider templates.
- Safer defaults: generated Basic Auth password; same-origin unless CORS is explicitly allowed.
- Configurable `HOST`/`PORT` for reverse proxies/TLS.
- Backup auto-failover: switch to a backup provider when relays fail.

## Highlights
- Provider switching with live sync (Claude/Codex/Gemini).
- Unified MCP management (import/export across clients).
- Skills marketplace with repo scanning and one-click install.
- Prompt management with CodeMirror editor.
- Import/export with backups; directory overrides for WSL/cloud sync.

## Quick Start (Web)
```bash
pnpm install
pnpm build:web
cd src-tauri
cargo build --release --features web-server --bin cc-switch-server
HOST=0.0.0.0 PORT=3000 ./target/release/cc-switch-server
```
- Login: `admin` / password in `~/.cc-switch/web_password` (auto-generated).
- CORS: same-origin by default; set `CORS_ALLOW_ORIGINS` (optional `CORS_ALLOW_CREDENTIALS=true`) to allow cross-origin.
- Web mode does not support native file/directory pickers—enter paths manually.

## Screenshots
| Skills marketplace | Prompt editor | Advanced settings |
| :--: | :--: | :--: |
| ![Skills](assets/screenshots/web-skills.png) | ![Prompt](assets/screenshots/web-prompt.png) | ![Settings](assets/screenshots/web-settings.png) |

*(Place your screenshots at the above paths.)*

## Web Server Notes
- File/dir pickers return 501 in web mode; use manual paths.
- Same-origin by default; use `CORS_ALLOW_ORIGINS` (+ `CORS_ALLOW_CREDENTIALS=true` if needed) for cross-origin.

## Project Structure (key parts)
- `src/` React/TypeScript frontend
- `src-tauri/` Rust backend (Tauri/web-server)
- `src-tauri/src/web_api/` Axum HTTP API for web mode
- `dist-web/` Built web assets (not committed)
- `tests/` Bash + MSW + Vitest tests

## Usage (common commands)
- Dev (desktop renderer): `pnpm dev:renderer`
- Build web assets: `pnpm build:web`
- Run web server: `HOST=0.0.0.0 PORT=3000 ./src-tauri/target/release/cc-switch-server`
- Build server: `cd src-tauri && cargo build --release --features web-server --bin cc-switch-server`
- Full test suite (bash APIs): `bash tests/run-all.sh` (needs running server & curl/jq)

## Tech Stack
- Frontend: React 18, TypeScript, Vite, Tailwind, TanStack Query, Radix UI, CodeMirror
- Backend: Rust, Axum, Tauri (for shared logic), tower-http
- Tooling: pnpm, Vitest, MSW, Bash + curl/jq for API tests

## Tests & Coverage
- API/integration bash tests under `tests/api` and `tests/integration` (providers, settings, MCP, usage, persistence).
- MSW/Vitest component/hooks tests for UI logic.
- Run `bash tests/run-all.sh` for web API coverage (requires running server).

## Changelog
See [CHANGELOG.md](CHANGELOG.md) (current version: v0.1.0).

## Project Highlights (recap)
- Web/server native (headless friendly), safe defaults (Basic Auth + same-origin).
- Unified provider/MCP/skills/prompt management across Claude/Codex/Gemini.
- Rich presets and backup auto-failover.
- Import/export with backups; WSL/cloud directory overrides.

## About the upstream project
This fork builds on **cc-switch** by Jason Young (farion1231). The upstream Tauri desktop app unified provider switching, MCP management, skills, and prompts with strong i18n and safety. cc-switch-web reuses that foundation, adds a web/server runtime, CORS controls, Basic Auth by default, more templates, and docs for cloud/headless deployment. Many thanks to the original author and contributors for the groundwork.

## Maintenance note
This is a newly published web/headless variant; some areas may still need polish. Please file issues for bugs or feature ideas. I’ll focus on updates during the coming week after seeing new issues, then shift to a weekly maintenance cadence until it’s solid for cloud development.
