# CC Switch v0.9.1

> `0.9.x` bugfix release notes

---

## 📦 What's New in v0.9.1 (2026-03-11)

This release focuses on **regression fixes for the `0.9.x` line**, especially around live-config behavior and relay-pulse compatibility.

`v0.8.0` remains the recommended stable release. If you are already validating `0.9.x`, upgrade to `v0.9.1`.

### ✅ Key Fixes

- **Fix current-provider save/apply not rewriting live config immediately**
  - Saving the active provider now updates the real live config files right away
- **Fix Codex current-provider updates overwriting MCP entries**
  - Enabled MCP entries such as `relay-pulse` are now preserved when updating the active provider
- **Support the current relay-pulse health payload**
  - Added compatibility for the new `groups[].layers[]` response shape instead of relying only on legacy `data[]`

### 🔧 UX Adjustments

- **Clarify the top app switcher behavior**
  - The top app switcher changes the management view only; it does not directly rewrite live config
  - Live config changes still happen when switching providers or saving the current active provider
- **Reduce startup noise**
  - Settings, prompts, MCP, and skills panels are now mounted lazily to keep the `0.9.x` debug flow cleaner

### 📌 Release Positioning

- **Recommended stable release**: `v0.8.0`
- **Bugfix preview release**: `v0.9.1`
- Stay on `v0.8.0` if you want the most conservative production recommendation
- Upgrade to `v0.9.1` if you are already testing or using the `0.9.x` line
