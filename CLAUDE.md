# Claude Code Launcher

タスクトレイ常駐型のClaude Codeランチャー（Windows向け）。

## 前提条件

- Windows 10/11
- `claude` コマンドがPATHに存在すること

## 開発コマンド

```bash
pnpm install        # 依存関係インストール
pnpm tauri dev      # 開発モード
pnpm tauri build    # 本番ビルド
pnpm bump <version> # バージョン一括更新
```

## 重要な同期ポイント

| 変更内容 | 更新が必要なファイル |
|----------|---------------------|
| バージョン更新 | package.json, Cargo.toml, tauri.conf.json（`pnpm bump`使用）。コミット後に `git tag v<version>` でタグを打ち、`git push origin v<version>` でプッシュする |
| Tauriコマンド追加 | src-tauri/src/lib.rs のみ（型定義は自動生成） |
| Rust型定義の変更 | `#[derive(ts_rs::TS)] #[ts(export)]`付き構造体を変更後、`pnpm ts:generate`で`src-tauri/bindings/`を更新し、必要に応じてsrc/types.tsから該当型をimportに置き換え |

## コードスタイル

**TypeScript/React (src/)**
- 関数コンポーネント + Hooks
- HashRouterでルーティング
- 共有型はtypes.tsに定義

**Rust (src-tauri/)**
- TauriコマンドはResult<T, String>で返す
- 型定義にts-rs deriveを追加することでTypeScript型を自動生成
- フロント共有型には `#[derive(ts_rs::TS)] #[ts(export)]` を付与
- `#[serde(rename_all = "camelCase")]` がある場合は `#[ts(rename_all = "camelCase")]` も追加
- `#[serde(rename = "foo")]` がある場合は `#[ts(rename = "foo")]` も追加

## 自動チェック

以下のチェックはClaude応答完了時（Stopフック）に自動実行される（手動実行不要）:

- TypeScript: `pnpm ts:typecheck` + `pnpm ts:lint` + `pnpm ts:fmt:check`
- JavaScript: `pnpm js:lint` + `pnpm js:fmt:check`
- Rust: `pnpm rs:lint` + `pnpm rs:fmt:check`

## 動作確認

**テスト実行**
```bash
pnpm test:all       # 全テスト一括（フロントエンド + Rust）
pnpm test           # フロントエンドテストのみ (Vitest)
pnpm hook:final     # 静的解析全チェック
pnpm ts:generate    # Rust型からTypeScript型定義を生成
```

**UI確認（playwright-cli）**

`pnpm tauri dev` 起動後、`playwright-cli` でTauri APIモックを注入してブラウザ上で確認する。
詳細手順: [docs/ui-check.md](docs/ui-check.md)

## 設定

設定ファイル: `%APPDATA%\cc-launcher\config.json`（型定義: `src/types.ts` の `AppConfig`）

## コード構成

| 関心事 | 参照先 |
|--------|--------|
| ウィンドウ定義 | `src-tauri/tauri.conf.json` |
| Tauriコマンド | `src-tauri/src/lib.rs`（`generate_handler![]`） |
| フロントエンドルート | `src/main.tsx` |
| TypeScript型定義 | `src-tauri/bindings/`（自動生成） + `src/types.ts`（手動定義） |
| Rust型定義 | `src-tauri/src/models.rs`, `src-tauri/src/config.rs` |
