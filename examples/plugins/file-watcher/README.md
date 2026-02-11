# file-watcher プラグイン

指定ディレクトリのファイル追加・変更を検知してイベントを発火するサンプルプラグイン。

## 必要環境

- Node.js（外部パッケージ不要）

## 使い方

### 直接実行で動作確認

```bash
node examples/plugins/file-watcher/index.cjs --dir C:\path\to\watch
```

起動後、stdinに以下を入力してinitialize:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

別ターミナルで監視対象ディレクトリにファイルを追加・変更すると、stdoutにイベントが出力される。

### cc-launcherへの登録

Manager画面のPluginsタブで以下を設定:

| 項目 | 値 |
|------|-----|
| Name | file-watcher |
| Executable | `node` |
| Args | `["C:\\path\\to\\examples\\plugins\\file-watcher\\index.cjs", "--dir", "C:\\target\\dir"]` |

### サブスクリプション設定例

Manager画面のSubscriptionsタブで以下を設定:

| 項目 | 値 |
|------|-----|
| Plugin Name | file-watcher |
| Event Type | `file_created` |
| Prompt Template | `新しいファイルが作成されました: {{file_path}}` |
| Working Dir | `C:\your\project` |

## コマンドライン引数

| 引数 | 説明 | デフォルト |
|------|------|-----------|
| `--dir <path>` | 監視対象ディレクトリ（必須） | — |
| `--ignore <patterns>` | 無視するパターン（カンマ区切り） | `.git,node_modules,.DS_Store,Thumbs.db` |
| `--debounce <ms>` | デバウンス間隔（ミリ秒） | `300` |

## イベント

### `file_created`

新規ファイル検知時に発火。

```json
{
  "eventType": "file_created",
  "data": {
    "file_path": "C:\\watched\\dir\\new-file.txt",
    "event_type": "file_created",
    "timestamp": "2025-01-01T00:00:00.000Z"
  }
}
```

### `file_changed`

既存ファイル変更時に発火。

```json
{
  "eventType": "file_changed",
  "data": {
    "file_path": "C:\\watched\\dir\\existing-file.txt",
    "event_type": "file_changed",
    "timestamp": "2025-01-01T00:00:00.000Z"
  }
}
```
