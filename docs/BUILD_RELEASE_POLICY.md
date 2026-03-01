# Build & Release Policy / 构建与发布策略

## EN

1. Do **not** produce release artifacts on this server.
2. All release binaries/packages must be built by GitHub Actions workflows (CI + Release).
3. Local outputs are only for temporary verification and must not be committed:
   - `src-tauri/target/`
   - `dist-web/`
   - ad-hoc binaries
   - `coverage/`
4. Official release flow:
   - push code
   - let CI validate
   - create/push release tag
   - let Release workflow build and upload artifacts

## 中文

1. **禁止**在当前服务器上生成正式发布产物。
2. 所有发布二进制/安装包必须通过 GitHub Actions（CI + Release）构建。
3. 本地构建产物仅用于临时验证，且禁止提交到仓库，包括但不限于：
   - `src-tauri/target/`
   - `dist-web/`
   - 临时二进制文件
   - `coverage/`
4. 正式发布流程：
   - 推送代码
   - 等待 CI 校验通过
   - 创建并推送 release tag
   - 由 Release 工作流完成构建与上传
