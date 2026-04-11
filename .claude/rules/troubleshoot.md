# トラブルシュート原則

## 定量基準

- **同じコマンドの失敗が 2 回続いたら止まる**。3 回目を実行する前にユーザーに現状報告し、原因推定と対処案を提示して承認を得る。
- **環境系コマンドは失敗 1 回で止まる**。対象: `pnpm install`, `pnpm store prune`, `rm -rf`, `Remove-Item`, `git reset --hard`, `git clean -f` など依存関係管理・ファイルシステム破壊を伴うコマンド。これらは `.claude/settings.local.json` の `ask` に登録されており、ユーザー承認を経てから実行される。
- **型チェック・lint・format・テスト由来のエラーは 3 回まで修正試行可**。それ以上ループしそうなら止まってユーザー相談。

## commit 失敗時の切り分け手順

pre-commit フックで失敗したら、lefthook の出力を読み直すよりも**個別コマンドで切り分ける方が速い**:

1. `pnpm ts:fmt:check` — フォーマット差分
2. `pnpm ts:typecheck` — 型エラー
3. `pnpm ts:lint` — lint エラー
4. `pnpm test` — テスト失敗

commit メッセージは `git commit -m "..."` + heredoc で渡す。tmp ファイル経由は運用外。
