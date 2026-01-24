# Claude Code Launcher

タスクトレイ常駐型のClaude Codeランチャーアプリケーション。

グローバルショートカットでどこからでもClaude Codeを呼び出せるWindows向けランチャー。

## 主要機能

- **タスクトレイ常駐**: バックグラウンドで動作し、システムトレイにアイコン表示
- **グローバルショートカット**: `Ctrl+Shift+Space`（カスタマイズ可能）で入力欄を表示
- **ターミナル自動検出**: pwsh > powershell > cmd の優先度で最適なターミナルを選択
- **設定UI**: トレイメニューからショートカットとターミナルを設定可能

## 前提条件

- Windows 10/11
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) がインストール済みで、`claude` コマンドがPATHに存在すること

## インストール

[Releases](https://github.com/fuchigta/cc-launcher/releases)からインストーラー（.msi）をダウンロードして実行してください。

## 操作方法

| 操作 | 動作 |
|------|------|
| `Ctrl+Shift+Space` | 入力欄の表示/非表示トグル |
| `Enter` | 入力したプロンプトでclaudeコマンドを実行 |
| `ESC` | 入力欄を閉じる |
| トレイアイコン左クリック | 入力欄の表示/非表示トグル |
| トレイアイコン右クリック | メニュー表示（Show Input / Settings / Quit） |

## 設定

設定は `%APPDATA%\cc-launcher\config.json` に保存されます。

| 項目 | 説明 |
|------|------|
| shortcut | グローバルショートカット（デフォルト: `Ctrl+Shift+Space`） |
| terminal | 使用するターミナル（Auto / Pwsh / PowerShell / Cmd） |

## 技術スタック

- **フロントエンド**: React, TypeScript, Vite
- **バックエンド**: Rust, Tauri 2.0

## 開発

```bash
pnpm install     # 依存関係インストール
pnpm tauri dev   # 開発モード
pnpm tauri build # 本番ビルド
```

## ライセンス

MIT
