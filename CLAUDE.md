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
| バージョン更新 | package.json, Cargo.toml, tauri.conf.json（`pnpm bump`使用） |
| Tauriコマンド追加 | src-tauri/src/lib.rs + src/types.ts |

## コードスタイル

**TypeScript/React (src/)**
- 関数コンポーネント + Hooks
- HashRouterでルーティング
- 共有型はtypes.tsに定義

**Rust (src-tauri/)**
- TauriコマンドはResult<T, String>で返す
- コマンド追加時はtypes.tsも更新

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
```

**UI確認（playwright-cli）**

`pnpm tauri dev` 起動後、`playwright-cli` でTauri APIモックを注入してブラウザ上で確認する。
詳細手順: [docs/ui-check.md](docs/ui-check.md)

## 設定

設定ファイル: `%APPDATA%\cc-launcher\config.json`

| 項目 | 選択肢 |
|------|--------|
| shortcut | キーコンビネーション（デフォルト: Ctrl+Shift+Space） |
| terminal | Auto, Pwsh, PowerShell, Cmd |

## ウィンドウ・コマンド

| ウィンドウ | 用途 |
|------------|------|
| main | 入力オーバーレイ（透明、常に最前面） |
| settings | 設定画面 |

| Tauriコマンド | 説明 |
|---------------|------|
| get_config / save_config | 設定の取得・保存 |
| get_available_terminals | 利用可能なターミナル一覧 |
| open_claude_interactive | claudeコマンドをターミナルで起動 |
| hide_window | ウィンドウを非表示 |
