# Claude Code Launcher

タスクトレイ常駐型のClaude Codeランチャーアプリケーション。

## 概要

グローバルショートカットでどこからでもClaude Codeを呼び出せるWindows向けランチャー。Tauri 2.0 + React + TypeScriptで構築。

## 主要機能

- **タスクトレイ常駐**: バックグラウンドで動作し、システムトレイにアイコン表示
- **グローバルショートカット**: `Ctrl+Shift+Space`（カスタマイズ可能）で入力欄を表示
- **ターミナル自動検出**: pwsh > powershell > cmd の優先度で最適なターミナルを選択
- **設定UI**: トレイメニューからショートカットとターミナルを設定可能

## プロジェクト構造

```
cc-launcher/
├── src/                      # フロントエンド (React + TypeScript)
│   ├── main.tsx              # エントリーポイント（HashRouterでルーティング）
│   ├── App.tsx               # 入力オーバーレイUI
│   ├── Settings.tsx          # 設定画面
│   ├── App.css               # スタイル
│   └── types.ts              # 共有型定義
├── src-tauri/                # バックエンド (Rust + Tauri 2.0)
│   ├── src/
│   │   ├── main.rs           # Rustエントリーポイント
│   │   ├── lib.rs            # メインロジック（トレイ、ショートカット、コマンド）
│   │   ├── config.rs         # 設定ファイル管理
│   │   └── terminal.rs       # ターミナル検出・起動
│   ├── Cargo.toml            # Rust依存関係
│   ├── tauri.conf.json       # Tauri設定
│   └── capabilities/
│       └── default.json      # 権限設定
├── index.html
├── package.json
└── CLAUDE.md
```

## 技術スタック

- **フロントエンド**: React 18, TypeScript, React Router 7
- **バックエンド**: Rust, Tauri 2.0
- **Tauriプラグイン**:
  - `tauri-plugin-global-shortcut`: グローバルホットキー
  - `tauri-plugin-shell`: ターミナル起動
  - `tauri-plugin-opener`: 外部リンク

## 開発コマンド

```bash
# 依存関係インストール
pnpm install

# 開発モードで起動
pnpm tauri dev

# 本番ビルド
pnpm tauri build
```

## バージョン更新

3ファイル（package.json, Cargo.toml, tauri.conf.json）のバージョンを一括更新する。

```bash
pnpm bump <version>

# 例
pnpm bump 1.0.0
pnpm bump 1.0.0-beta.1
```

## 設定ファイル

設定は `%APPDATA%\cc-launcher\config.json` に保存される。

```json
{
  "shortcut": "Ctrl+Shift+Space",
  "terminal": "Auto"
}
```

**terminal の選択肢:**
- `Auto`: 自動検出（pwsh優先）
- `Pwsh`: PowerShell 7+
- `PowerShell`: Windows PowerShell
- `Cmd`: Command Prompt

## ウィンドウ構成

| ラベル | 用途 | 特徴 |
|--------|------|------|
| `main` | 入力オーバーレイ | 透明、装飾なし、常に最前面 |
| `settings` | 設定画面 | 通常ウィンドウ |

## Tauriコマンド

| コマンド | 説明 |
|----------|------|
| `get_config` | 現在の設定を取得 |
| `save_config` | 設定を保存 |
| `get_available_terminals` | 利用可能なターミナル一覧 |
| `open_claude_interactive` | claudeコマンドをターミナルで起動 |
| `hide_window` | ウィンドウを非表示 |

## 操作方法

- **Ctrl+Shift+Space**: 入力欄の表示/非表示トグル
- **Enter**: 入力したプロンプトでclaudeコマンドを実行
- **ESC**: 入力欄を閉じる
- **トレイアイコン左クリック**: 入力欄の表示/非表示トグル
- **トレイアイコン右クリック**: メニュー表示（Show Input / Settings / Quit）

## 前提条件

- `claude` コマンドがPATHに存在すること
- Windows 10/11
