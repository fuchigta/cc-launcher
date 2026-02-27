#!/bin/bash
INPUT=$(cat)

# 無限ループ防止: フック再入時は即終了
[ "$(echo "$INPUT" | jq -r '.stop_hook_active')" = "true" ] && exit 0

if ! pnpm --prefix "$(dirname "$0")/../.." hook:final 2>&1; then
  echo "静的解析チェックに失敗しました。上記のエラーを修正してください。" >&2
  exit 2
fi
