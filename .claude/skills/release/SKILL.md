---
name: release
description: This skill should be used when the user asks to "リリースしてください", "タグ打ってプッシュ", "バージョンを上げてリリース", "release", "bump version and push", or wants to publish a new version of cc-launcher. Guides through the full release workflow: commit pending changes, bump version, commit the bump, create a git tag, and push.
version: 0.1.0
---

# cc-launcher リリース手順

## 概要

cc-launcher のリリースは以下の順序で行う:

1. 未コミットの変更をコミット
2. `pnpm bump <version>` でバージョン一括更新
3. バージョン更新をコミット
4. `pnpm tag` でタグ作成 & push

## 重要な制約

- **git-cliff の制約**: `pnpm bump` 実行時に未コミットの変更があると CHANGELOG.md に反映されない。実装変更は必ず先にコミットすること。
- **バージョン形式**: semver（例: `0.17.16`, `1.0.0`）
- **バージョン更新コミットメッセージ**: `chore: バージョンを<version>に更新`（このプロジェクトの慣例）

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

### Step 3: 新バージョンを確認してユーザーに提案

現在のバージョンから次のバージョンを提案し、ユーザーに確認を取る。通常はパッチバージョンを +1（例: `0.17.15` → `0.17.16`）。

ユーザーの承認を得てから次のステップへ進む。

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
