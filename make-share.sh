#!/usr/bin/env bash
# 打包可分享的项目压缩包:只含源码/界面母本/安装包,不含运行时、依赖、构建缓存和私人内容
set -euo pipefail
cd "$(dirname "$0")"

STAGE="$(mktemp -d)/LynceAI"
mkdir -p "$STAGE"
OUT="$(pwd)/../LynceAI-share-$(date +%Y%m%d).zip"
rm -f "$OUT"

# 顶层公共文件
cp README.md .gitignore "$STAGE/"

# 启动器源码(排除依赖/构建缓存/私人内容)
rsync -a \
  --exclude node_modules --exclude 'src-tauri/target' --exclude dist \
  --exclude .DS_Store \
  aki-mac-launcher/ "$STAGE/aki-mac-launcher/"

# v0.2 安装包(releases/ 为常驻保存位置;若刚构建过 target 里也有,优先取新的)
mkdir -p "$STAGE/aki-mac-launcher/release"
cp aki-mac-launcher/releases/*.dmg "$STAGE/aki-mac-launcher/release/" 2>/dev/null || \
  cp aki-mac-launcher/src-tauri/target/release/bundle/dmg/*.dmg "$STAGE/aki-mac-launcher/release/" 2>/dev/null || true

# 网页界面母本
rsync -a --exclude .DS_Store liblib-ui/ "$STAGE/liblib-ui/"

(cd "$(dirname "$STAGE")" && zip -qr "$OUT" LynceAI)
rm -rf "$(dirname "$STAGE")"

echo "✓ 分享包已生成: $OUT ($(du -h "$OUT" | cut -f1))"
unzip -l "$OUT" | tail -3
