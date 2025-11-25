# CC-Switch Web Server 使用指南

## 概述

CC-Switch 现在支持两种运行模式：
1. **Tauri GUI 模式** - 原有的桌面应用（Windows/macOS/Linux）
2. **Web Server 模式** - 新增的无头服务器模式（适用于云服务器）

Web Server 模式完整保留了所有原有功能，通过浏览器访问可视化界面。

---

## 快速开始

### 1. 构建 Web 前端

```bash
cd /root/cc-switch
pnpm install
pnpm build:web
```

这会在 `dist-web/` 目录生成静态文件。

### 2. 编译服务器

```bash
cd src-tauri
cargo build --release --features web-server --bin cc-switch-server
```

生成的可执行文件位于：`target/release/cc-switch-server`

### 3. 运行服务器

```bash
# 默认监听 0.0.0.0:3000（可用 HOST/PORT 覆盖）
./target/release/cc-switch-server

# 自定义地址与端口
HOST=0.0.0.0 PORT=8080 ./target/release/cc-switch-server
```

如需跨域访问，可按需设置：

```bash
# 允许的 Origin 列表（逗号分隔），不设置则仅同源可用
CORS_ALLOW_ORIGINS=https://example.com,https://admin.example.com \
CORS_ALLOW_CREDENTIALS=true \
./target/release/cc-switch-server
```

### Web 模式限制

- 系统文件/目录选择器在 Web 模式不可用（相关 API 返回 501），需要手动输入路径。
- 默认仅同源可访问；开启跨域需设置 `CORS_ALLOW_ORIGINS`（可选 `CORS_ALLOW_CREDENTIALS=true`）。

### 4. 访问 Web 界面

浏览器访问：`http://your-server-ip:3000`

---

## 安全与认证（生产必读）

- **强制设置凭证**：部署前请设置环境变量：
  - `WEB_API_TOKEN=<随机长 token>`：服务端 Bearer Token（默认等同于密码，不建议沿用）。
  - `WEB_CSRF_TOKEN=<随机短 token>`：前端请求需在非 GET/HEAD 请求头携带 `X-CSRF-Token`。
- **自动生成**：若上述环境变量缺失，服务会自动生成随机 Token 写入 `~/.cc-switch/web_env`，并在前端注入 `window.__CC_SWITCH_TOKENS__`，无须手工输入即可完成认证。
- **前端提示**：界面会显示“已自动应用凭证”徽标，表示前端已自动携带 Authorization/CSRF 头，无需额外操作。
- **HTTPS 反代**：建议用 Nginx/Caddy/Cloudflare 等做 TLS 终止，把 cc-switch-server 放在反代后面。
- **HSTS**：默认开启 `Strict-Transport-Security`，如需关闭可设 `ENABLE_HSTS=false`。
- **裸 HTTP 风险**：若必须在无 TLS 的公网监听，需显式设置 `ALLOW_HTTP_BASIC_OVER_HTTP=1` 表示接受风险；否则请保持在内网/回环地址。
- **跨域**：默认同源，若确需跨域，使用 `CORS_ALLOW_ORIGINS=https://foo.com,https://bar.com`（不要使用 `*`）。

运行示例（反代模式，含凭证）：

```bash
WEB_API_TOKEN="$(openssl rand -hex 32)" \
WEB_CSRF_TOKEN="$(openssl rand -hex 16)" \
HOST=127.0.0.1 PORT=3000 ./target/release/cc-switch-server
```

前端在非 GET/HEAD 请求中需要附带：

```
Authorization: Bearer <WEB_API_TOKEN>
X-CSRF-Token: <WEB_CSRF_TOKEN>
```

---

## 架构说明

```
┌─────────────────────────────────────────────────┐
│         cc-switch-server (单个可执行文件)        │
│  ┌─────────────────────────────────────────┐   │
│  │  HTTP Server (axum)                     │   │
│  │  - 静态文件 (React UI)                  │   │
│  │  - REST API (/api/*)                    │   │
│  └─────────────────┬───────────────────────┘   │
│                    │                            │
│  ┌─────────────────▼───────────────────────┐   │
│  │  Services Layer (复用桌面版代码)        │   │
│  │  ProviderService | McpService | ...     │   │
│  └─────────────────┬───────────────────────┘   │
│                    │                            │
│  ┌─────────────────▼───────────────────────┐   │
│  │  Config Files (~/.cc-switch/...)        │   │
│  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

---

## API 端点

### Provider 管理

```
GET    /api/providers/:app             # 获取所有供应商
GET    /api/providers/:app/current     # 获取当前供应商
POST   /api/providers/:app             # 添加供应商
PUT    /api/providers/:app/:id         # 更新供应商
DELETE /api/providers/:app/:id         # 删除供应商
POST   /api/providers/:app/:id/switch  # 切换供应商
```

`:app` 可选值：`claude`, `codex`, `gemini`

### MCP 管理

```
GET    /api/mcp/servers      # 获取 MCP 服务器列表
POST   /api/mcp/servers      # 添加 MCP 服务器
PUT    /api/mcp/servers/:id  # 更新 MCP 服务器
DELETE /api/mcp/servers/:id  # 删除 MCP 服务器
```

### Settings 管理

```
GET    /api/settings   # 获取设置
PUT    /api/settings   # 保存设置
```

### Config 导入/导出

```
POST   /api/config/export  # 导出配置
POST   /api/config/import  # 导入配置
```

---

## 部署到云服务器

### 方案一：直接运行

```bash
# 1. 上传编译好的二进制文件
scp target/release/cc-switch-server user@server:/usr/local/bin/

# 2. SSH 到服务器运行
ssh user@server
/usr/local/bin/cc-switch-server

# 3. 后台运行（使用 nohup）
nohup /usr/local/bin/cc-switch-server > /var/log/cc-switch.log 2>&1 &
```

### 方案二：Systemd 服务

创建 `/etc/systemd/system/cc-switch.service`:

```ini
[Unit]
Description=CC-Switch Web Server
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user
ExecStart=/usr/local/bin/cc-switch-server
Environment=PORT=3000
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

启用服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable cc-switch
sudo systemctl start cc-switch
sudo systemctl status cc-switch
```

### 方案三：使用 screen/tmux

```bash
screen -S cc-switch
/usr/local/bin/cc-switch-server
# Ctrl+A D 分离会话

# 重新连接
screen -r cc-switch
```

---

## 配置文件位置

所有配置文件与桌面版完全相同：

- **主配置**: `~/.cc-switch/config.json`
- **Settings**: `~/.cc-switch/settings.json`
- **Backups**: `~/.cc-switch/backups/`
- **Claude**: `~/.claude/settings.json`, `~/.claude.json`
- **Codex**: `~/.codex/auth.json`, `~/.codex/config.toml`
- **Gemini**: `~/.gemini/.env`, `~/.gemini/settings.json`

---

## 特性说明

### 完整功能映射

Web 端完全复制了桌面端的所有功能：

1. ✅ Provider 管理（添加/编辑/删除/切换）
2. ✅ MCP 服务器管理
3. ✅ Settings 配置
4. ✅ Config 导入/导出
5. ✅ 端点速度测试
6. ✅ Claude/Codex/Gemini 三者支持

### 自动环境检测

前端代码会自动检测运行环境：
- **Tauri 环境**: 使用 IPC 通信
- **Web 环境**: 使用 HTTP API

### CORS 支持

开发模式下启用了 `CorsLayer::very_permissive()`，生产环境建议配置反向代理限制来源。

---

## 安全建议

### 1. 反向代理（推荐）

使用 Nginx/Caddy 添加 HTTPS：

```nginx
server {
    listen 443 ssl;
    server_name cc-switch.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 2. 防火墙

仅允许特定 IP 访问：

```bash
# UFW 示例
sudo ufw allow from 192.168.1.0/24 to any port 3000
```

### 3. SSH 隧道

本地开发时使用 SSH 隧道：

```bash
ssh -L 3000:localhost:3000 user@server
# 访问 http://localhost:3000
```

---

## 故障排查

### 编译错误

```bash
# 确保安装了所有依赖
apt-get install libglib2.0-dev libgtk-3-dev

# 清理并重新构建
cargo clean
cargo build --release --features web-server --bin cc-switch-server
```

### 前端构建失败

```bash
# 清理缓存
rm -rf node_modules dist-web
pnpm install
pnpm build:web
```

### 服务器启动失败

```bash
# 检查端口占用
lsof -i:3000

# 查看日志
RUST_LOG=debug ./cc-switch-server
```

### 浏览器无法访问

1. 检查防火墙设置
2. 确认服务器监听 `0.0.0.0` 而非 `127.0.0.1`
3. 检查云服务器安全组规则

---

## 性能优化

### 1. Release 模式编译

```bash
cargo build --release --features web-server --bin cc-switch-server
```

### 2. 静态文件压缩

前端构建已启用 Vite 的压缩优化。

### 3. 资源限制

```bash
# 限制内存使用（systemd）
[Service]
MemoryLimit=512M
```

---

## 开发模式

### 前端开发

```bash
# 启动 Vite 开发服务器（自动代理到后端）
pnpm dev:web
```

- 开发端口已调整为 **4173**，并默认将 `/api` 代理到 `http://localhost:3000`，避免与 cc-switch-server 默认端口冲突。

### 后端开发

```bash
# 监听文件变化自动重新编译
cargo watch -x 'run --features web-server --bin cc-switch-server'
```

---

## 迁移指南

### 从桌面版迁移

1. 配置文件完全兼容，无需修改
2. 直接在服务器上运行 `cc-switch-server`
3. 访问 Web 界面继续使用

### 复制到其他服务器

```bash
# 1. 打包二进制文件
tar -czf cc-switch-server.tar.gz target/release/cc-switch-server

# 2. 上传到新服务器
scp cc-switch-server.tar.gz user@new-server:/tmp/

# 3. 解压并运行
ssh user@new-server
cd /tmp
tar -xzf cc-switch-server.tar.gz
mv target/release/cc-switch-server /usr/local/bin/
cc-switch-server
```

---

## 常见问题

**Q: 是否可以同时运行桌面版和 Web 版？**
A: 可以，它们共享同一份配置文件。但不建议同时操作，可能导致配置冲突。

**Q: Web 版性能如何？**
A: 由于复用了相同的 Rust 后端代码，性能与桌面版几乎一致。

**Q: 需要安装 Node.js 吗？**
A: 服务器运行不需要。只有构建前端时需要 Node.js + pnpm。

**Q: 如何更新？**
A: 重新编译并替换二进制文件，重启服务即可。

---

## 技术栈

- **前端**: React 18 + TypeScript + TailwindCSS + TanStack Query
- **后端**: Axum (Rust) + Tokio + Rust-Embed
- **通信**: REST API (HTTP) / Tauri IPC（桌面版）

---

## 贡献

欢迎提交 Issue 和 Pull Request！

Web Server 模式的代码位于：
- `/src-tauri/src/web_api/` - HTTP API 实现
- `/src-tauri/src/bin/server.rs` - 服务器入口
- `/src/lib/api/adapter.ts` - 前端适配器
- `/vite.config.web.mts` - Web 构建配置

---

## License

MIT © Jason Young
