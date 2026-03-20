---
name: release
description: >-
  This skill should be used when the user asks to "リリースしてください",
  "タグ打ってプッシュ", "バージョンを上げてリリース", "release",
  "bump version and push", "CIを監視して", "CI確認して",
  "リリース結果を確認", "watch CI", "monitor CI",
  or wants to publish a new version of cc-launcher.
  Guides through the full release workflow: commit pending changes,
  bump version, commit the bump, create a git tag, push,
  then monitor CI/Release workflows and fix failures if needed.
version: 0.2.0
---

# cc-launcher リリース手順

## 概要

cc-launcher のリリースは以下の順序で行う:

1. 未コミットの変更をコミット
2. `pnpm bump <version>` でバージョン一括更新
3. バージョン更新をコミット
4. `pnpm tag` でタグ作成 & push
5. GitHub Actions の `Release` ワークフロー完了を確認
6. 失敗時: 分析・修正・再リリース
7. 成功確認 & リリースURL報告

## 重要な制約

- **git-cliff の制約**: `pnpm bump` 実行時に未コミットの変更があると CHANGELOG.md に反映されない。実装変更は必ず先にコミットすること。
- **バージョン形式**: semver（例: `0.17.16`, `1.0.0`）
- **バージョン更新コミットメッセージ**: `chore: バージョンを<version>に更新`（このプロジェクトの慣例）
- **CI監視の制約**: 修正→再リリースは最大2回まで。インフラ起因の失敗は `gh run rerun --failed` で再実行（回数制限なし）。

## 手順

### Step 1: 現状確認

```bash
git status
git log --oneline -5
grep '^version' src-tauri/Cargo.toml
```

未コミットの変更の有無と現在のバージョンを把握する。

### Step 2: 未コミットの変更をコミット

未コミットの変更がある場合、変更内容に応じた prefix でコミットする。

| prefix | 用途 |
|---|---|
| `feat:` | 新機能 |
| `fix:` | バグ修正 |
| `chore:` | ビルド・設定変更 |
| `refactor:` | リファクタリング |
| `test:` | テスト追加・修正 |
| `docs:` | ドキュメント |

```bash
git add <files>
git commit -m "feat: ..."
```

コミット後、Step 3 へ進む前に `git status` でクリーンなことを確認する。

### Step 3: 次バージョンを決定する

git log でコミット内容を確認し、semver のルールに従って次バージョンを自動決定する。

```bash
git log <current_version_tag>..HEAD --oneline
```

| コミット内訳 | バージョン判定 | 確認 |
|---|---|---|
| `fix:` / `chore:` / `test:` / `docs:` のみ | パッチ +1 | **不要** |
| `feat:` を含む | マイナー +1 | **不要** |
| `BREAKING CHANGE` を含む | メジャー +1 | **不要** |
| 判断が難しい・複数ルールが競合する | — | `AskUserQuestion` で確認 |

判断できる場合はそのまま Step 4 へ進む。判断できない場合のみ `AskUserQuestion` ツールで確認を取る。

### Step 4: `pnpm bump <version>`

以下のファイルが自動更新される:

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`（cargo update）
- `.claude-plugin/plugin.json`
- `.claude-plugin/marketplace.json`
- `CHANGELOG.md`（git-cliff で自動生成）

```bash
pnpm bump <version>
```

### Step 5: バージョン更新をコミット

```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock .claude-plugin/plugin.json .claude-plugin/marketplace.json CHANGELOG.md
git commit -m "chore: バージョンを<version>に更新"
```

### Step 6: タグ作成 & push

```bash
pnpm tag
```

`v<version>` タグが作成され、`main` ブランチとタグが origin へ push される。

### Step 7: CI + Release ワークフローを個別に監視

タグ push により GitHub Actions の CI ワークフロー（`ci.yml`）と Release ワークフロー（`release.yml`）が起動する。**トークン消費を最小化するため、監視スクリプトを1回のツール呼び出しで実行する。**

```bash
# timeout: 600000 を指定すること（Bash ツールの上限 = 10分）
bash "<SKILL_BASE_DIR>/scripts/wait-workflows.sh"
```

- 終了コード `0`: 両方成功 → Step 8 へ
- 終了コード `1`: タイムアウト（まだ実行中）→ 同じコマンドを再実行
- 終了コード `2`: 失敗検出 → Step 8 へ

### Step 8: 結果判定

両ワークフローが完了したら結果を確認する:

```bash
gh run view <run-id> --log-failed
```

| 状態 | 起因 | 対応 |
|---|---|---|
| 両方 `success` | — | Step 10（完了報告）へ |
| `failure` でログにネットワーク/タイムアウト/rate-limit | インフラ起因 | `gh run rerun --failed <run-id>` で再実行（Step 7 へ戻る） |
| `failure` でログにコンパイルエラー/テスト失敗/設定ミス | コード起因 | Step 9（修正 & 再リリース）へ |

### Step 9: 修正 & 再リリース（最大2回）

コード起因の失敗の場合、修正してパッチバージョンで再リリースする。

1. 原因を特定・修正
2. 修正をコミット（`fix: ...`）
3. Step 3 に戻り次バージョン（パッチ +1）を決定
4. Step 4〜7 を実行

リトライ回数を記録し、2回目の失敗後は Step 10b へ進む。

### Step 10a: 成功確認 & リリースURL報告

```bash
gh release view --repo fuchigta/cc-launcher v<version>
```

GitHub Release が作成されていることを確認し、リリース URL をユーザーに報告して完了。

### Step 10b: リトライ上限到達

修正 & 再リリースを2回試みても失敗した場合、以下をユーザーに報告して終了する:

- 失敗したワークフローの Run ID とログ URL
- 失敗の概要（エラーメッセージ抜粋）
- 推奨アクション（手動調査 or 追加コンテキストの提供依頼）
