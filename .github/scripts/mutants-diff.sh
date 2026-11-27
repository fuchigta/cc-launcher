#!/usr/bin/env bash
# ミューテーションテストの比較元との差分（cargo-mutants の --in-diff に渡す）を標準出力に出す。
# 引数は比較元のコミット（pull_request は origin/<ベースブランチ>、push は push 前の SHA）。
# 比較元が無い（main を新しく作った push など）ときは、直前のコミットと比べる。
#
# パスは Cargo ワークスペースの根（src-tauri/）からの相対にする（--relative=src-tauri）。
# cargo-mutants は差分のパスを木の根からの相対として読むため、リポジトリの根からのパス
# （src-tauri/src/...）のままだと、エラーも出さずに 0 件になる（2026-11-27 に実測）。
set -euo pipefail
base="${1:-}"
if [[ -z "$base" || "$base" =~ ^0+$ ]] || ! git cat-file -e "${base}^{commit}" 2>/dev/null; then
  echo "比較元 '${base}' が見つからないため、HEAD^ と比べる" >&2
  base="HEAD^"
fi
echo "比較元: $(git rev-parse "$base")" >&2
git diff --relative=src-tauri "${base}...HEAD"
