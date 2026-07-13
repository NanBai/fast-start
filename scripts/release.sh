#!/usr/bin/env bash
# Session Launcher release helper.
# DRY_RUN=1 只打印步骤、不构建/不上传。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DRY_RUN="${DRY_RUN:-0}"

log() { printf '%s\n' "$*"; }
run() {
  if [[ "$DRY_RUN" == "1" ]]; then
    log "[dry-run] $*"
  else
    log "+ $*"
    eval "$@"
  fi
}

fail() { log "ERROR: $*"; exit 1; }

PKG_VERSION="$(node -p "require('./package.json').version")"
CARGO_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' src-tauri/Cargo.toml | head -1)"
TAURI_VERSION="$(node -p "const c=require('./src-tauri/tauri.conf.json'); c.version || c.package?.version || ''")"

log "package.json:     $PKG_VERSION"
log "Cargo.toml:       $CARGO_VERSION"
log "tauri.conf.json:  $TAURI_VERSION"

if [[ -z "$PKG_VERSION" || -z "$CARGO_VERSION" || -z "$TAURI_VERSION" ]]; then
  fail "无法读取版本号"
fi

if [[ "$PKG_VERSION" != "$CARGO_VERSION" || "$PKG_VERSION" != "$TAURI_VERSION" ]]; then
  fail "版本不一致：package=$PKG_VERSION cargo=$CARGO_VERSION tauri=$TAURI_VERSION"
fi

log "版本一致：$PKG_VERSION"

run "pnpm install"
run "pnpm build"
run "pnpm tauri build"

TAG="v${PKG_VERSION}"
DMG_GLOB="src-tauri/target/release/bundle/dmg/*.dmg"

log "下一步（人工门禁）："
log "  1. 如需公证/签名，请在本机完成后再创建 Release"
log "  2. 创建 GitHub Release 示例："
log "     gh release create ${TAG} ${DMG_GLOB} --title \"${TAG}\" --notes \"Session Launcher ${PKG_VERSION}\""

if [[ "$DRY_RUN" == "1" ]]; then
  log "[dry-run] 跳过 gh release create"
  exit 0
fi

if [[ "${CREATE_GITHUB_RELEASE:-0}" == "1" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    fail "未安装 gh CLI"
  fi
  # shellcheck disable=SC2086
  gh release create "${TAG}" ${DMG_GLOB} --title "${TAG}" --notes "Session Launcher ${PKG_VERSION}"
else
  log "未设置 CREATE_GITHUB_RELEASE=1，跳过 gh release create"
fi
