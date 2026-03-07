#!/bin/bash
msg=$(cat "$1")
pattern='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?(!)?: .+'
if ! echo "$msg" | grep -qE "$pattern"; then
  echo "コミットメッセージがConventional Commitsフォーマットに従っていません。"
  echo "例: feat: ログイン機能を追加"
  echo "    fix(auth): トークン期限切れの修正"
  echo "型: feat fix docs style refactor perf test build ci chore revert"
  exit 1
fi
