#!/usr/bin/env bash
# CC Switch Linux installer
# Usage: curl -fsSL https://raw.githubusercontent.com/farion1231/cc-switch/main/scripts/install.sh | bash

set -euo pipefail

REPO="${REPO:-farion1231/cc-switch}"
VERSION="${VERSION:-latest}"
ARCH="${ARCH:-$(uname -m)}"
NO_CHECKSUM="${NO_CHECKSUM:-0}"

log() { printf '\033[1;34m[ccswitch]\033[0m %s\n' "$*"; }
err() { printf '\033[1;31m[ccswitch]\033[0m %s\n' "$*" >&2; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    err "缺少命令: $1"
    exit 1
  }
}

for cmd in curl sha256sum; do
  need_cmd "$cmd"
done

case "$ARCH" in
  x86_64|amd64) ARCH=amd64 ;;
  aarch64|arm64) ARCH=arm64 ;;
  armv7l|armv7) ARCH=armv7 ;;
  *)
    err "暂不支持的架构: $ARCH"
    exit 1
    ;;
esac

if [[ "${EUID:-$(id -u)}" -eq 0 ]]; then
  PREFIX=/usr/local
  DESKTOP_DIR=/usr/share/applications
  ICON_DIR=/usr/local/share/icons/hicolor/512x512/apps
else
  PREFIX="${HOME}/.local"
  DESKTOP_DIR="${HOME}/.local/share/applications"
  ICON_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"
fi

BIN_DIR="${PREFIX}/bin"
INSTALL_PATH="${BIN_DIR}/ccswitch"
DESKTOP_FILE="${DESKTOP_DIR}/ccswitch.desktop"
ICON_PATH="${ICON_DIR}/ccswitch.png"

TMPDIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMPDIR"
}
trap cleanup EXIT

fetch_release() {
  local api_url
  if [[ "$VERSION" == "latest" ]]; then
    api_url="https://api.github.com/repos/${REPO}/releases/latest"
  else
    api_url="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
  fi

  curl -fsSL "$api_url"
}

select_asset() {
  local json="$1"
  python3 - "$ARCH" <<'PY'
import json, os, sys
arch = sys.argv[1]
data = json.load(sys.stdin)
assets = data.get("assets") or []
if not assets:
    sys.exit("no assets found in release metadata")

def score(name: str) -> int:
    name_l = name.lower()
    prefs = [
        f"{arch}.appimage",
        "appimage",
        f"{arch}.tar.gz",
        f"{arch}.tar.xz",
        "linux",
    ]
    for i, p in enumerate(prefs):
        if p in name_l:
            return 100 - i
    return 0

best = max(assets, key=lambda a: score(a["name"]))
if score(best["name"]) == 0:
    sys.exit("no suitable linux asset found")

checksum = None
for a in assets:
    name_l = a["name"].lower()
    if "sha256" in name_l and arch in name_l:
        checksum = a
        break
    if name_l.startswith("sha256") or name_l.endswith(".sha256"):
        checksum = a
        break

print(best["name"])
print(best["browser_download_url"])
if checksum:
    print(checksum["browser_download_url"])
PY
}

download_file() {
  local url="$1" out="$2"
  curl -fL "$url" -o "$out"
}

verify_checksum() {
  local file="$1" sum_file="$2"
  local hash_line expected
  hash_line=$(grep -Eo '([A-Fa-f0-9]{64})' "$sum_file" | head -n1 || true)
  if [[ -z "$hash_line" ]]; then
    err "校验文件中未找到 SHA256 值，跳过校验"
    return 1
  fi
  expected="$hash_line"
  printf '%s  %s\n' "$expected" "$file" | sha256sum -c -
}

install_appimage() {
  local payload="$1"
  install -d "$BIN_DIR"
  install -m 0755 "$payload" "$INSTALL_PATH"
}

install_from_tar() {
  local tarball="$1"
  local work="$TMPDIR/unpacked"
  mkdir -p "$work"
  tar -xf "$tarball" -C "$work"
  local bin
  bin=$(find "$work" -type f -perm -111 -maxdepth 3 \( -name "ccswitch" -o -name "cc-switch" -o -name "CCSwitch" -o -name "CC Switch" \) | head -n1 || true)
  if [[ -z "$bin" ]]; then
    bin=$(find "$work" -type f -perm -111 -maxdepth 3 | head -n1 || true)
  fi
  if [[ -z "$bin" ]]; then
    err "在压缩包中未找到可执行文件"
    exit 1
  fi
  install -d "$BIN_DIR"
  install -m 0755 "$bin" "$INSTALL_PATH"
}

write_desktop_entry() {
  install -d "$DESKTOP_DIR"
  cat >"$DESKTOP_FILE" <<EOF
[Desktop Entry]
Name=CC Switch
Exec=${INSTALL_PATH} %U
Icon=${ICON_PATH}
Type=Application
Categories=Utility;Development;
Terminal=false
EOF
}

install_icon() {
  install -d "$ICON_DIR"
  local icon_url="https://raw.githubusercontent.com/${REPO}/main/src-tauri/icons/icon.png"
  curl -fsSL "$icon_url" -o "$ICON_PATH" || err "下载图标失败，桌面快捷方式将使用默认图标"
}

main() {
  log "检测发行版信息 (${VERSION}, arch=${ARCH})"
  release_json="$(fetch_release)"
  mapfile -t lines < <(printf '%s' "$release_json" | select_asset)
  asset_name="${lines[0]}"
  asset_url="${lines[1]}"
  checksum_url="${lines[2]-}"

  log "准备下载: ${asset_name}"
  payload="$TMPDIR/$asset_name"
  download_file "$asset_url" "$payload"

  if [[ -n "${checksum_url:-}" && "$NO_CHECKSUM" != "1" ]]; then
    log "下载校验文件并验证 SHA256"
    checksum_file="$TMPDIR/sha256.txt"
    download_file "$checksum_url" "$checksum_file"
    verify_checksum "$payload" "$checksum_file" || {
      err "SHA256 校验失败"
      exit 1
    }
  else
    log "跳过校验 (未提供校验文件或 NO_CHECKSUM=1)"
  fi

  case "$asset_name" in
    *.AppImage|*.appimage)
      install_appimage "$payload"
      ;;
    *.tar.gz|*.tgz|*.tar.xz|*.tar.zst)
      install_from_tar "$payload"
      ;;
    *)
      err "未知的包格式: $asset_name"
      exit 1
      ;;
  esac

  install_icon || true
  write_desktop_entry

  log "已安装到: $INSTALL_PATH"
  if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    err "提示: 将 ${BIN_DIR} 加入 PATH 后可直接运行 ccswitch"
  fi
  log "桌面入口: $DESKTOP_FILE"
}

main "$@"
