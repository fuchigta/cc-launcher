# プラグインリファレンス

cc-launcher のプラグインシステム、組み込みプラグインのイベント仕様、カスタムプラグインの作成方法を説明します。

---

## プラグインアーキテクチャ概要

プラグインは **外部プロセス** として起動し、**JSON-RPC 2.0 形式** の stdin/stdout 通信で cc-launcher とイベントを交換します。

```
cc-launcher ──(initialize)──▶ plugin プロセス
cc-launcher ◀──(initialized)── plugin プロセス
cc-launcher ◀──(event 通知)──  plugin プロセス (繰り返し)
cc-launcher ──(shutdown)────▶  plugin プロセス
```

受信したイベントは **サブスクリプション** と照合され、マッチしたサブスクリプションのプロンプトテンプレートが展開されて Claude に送信されます。

---

## 組み込みプラグイン

### Folder Watcher

指定ディレクトリのファイル変更をリアルタイムに監視し、イベントを発火します。

#### コマンドライン引数

| 引数 | 説明 | デフォルト |
|------|------|-----------|
| `--dir <path>` | 監視対象ディレクトリ（**必須**） | — |
| `--recursive` | サブディレクトリも再帰的に監視 | `false` |
| `--filter <patterns>` | 対象ファイルのグロブパターン（カンマ区切り） | 全ファイル |
| `--ignore <patterns>` | 除外するパスコンポーネント（カンマ区切り） | `.git,node_modules` |
| `--debounce <ms>` | 変更検知のデバウンス間隔（ミリ秒） | `300` |

#### イベントカタログ

##### `file_created` — 新規ファイル作成

```json
{
  "eventType": "file_created",
  "data": {
    "file_path": "C:\\watched\\dir\\new-file.txt",
    "timestamp": "2025-01-01T09:00:00.000Z"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `file_path` | string | 作成されたファイルの絶対パス |
| `timestamp` | string (RFC3339) | イベント検知時刻（UTC） |

##### `file_changed` — ファイル内容変更

```json
{
  "eventType": "file_changed",
  "data": {
    "file_path": "C:\\watched\\dir\\existing.txt",
    "timestamp": "2025-01-01T09:00:00.000Z"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `file_path` | string | 変更されたファイルの絶対パス |
| `timestamp` | string (RFC3339) | イベント検知時刻（UTC） |

##### `file_deleted` — ファイル削除

```json
{
  "eventType": "file_deleted",
  "data": {
    "file_path": "C:\\watched\\dir\\deleted.txt",
    "timestamp": "2025-01-01T09:00:00.000Z"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `file_path` | string | 削除されたファイルの絶対パス |
| `timestamp` | string (RFC3339) | イベント検知時刻（UTC） |

##### `file_renamed` — ファイルリネーム

```json
{
  "eventType": "file_renamed",
  "data": {
    "old_path": "C:\\watched\\dir\\old-name.txt",
    "new_path": "C:\\watched\\dir\\new-name.txt",
    "timestamp": "2025-01-01T09:00:00.000Z"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `old_path` | string | リネーム前のファイルの絶対パス |
| `new_path` | string | リネーム後のファイルの絶対パス |
| `timestamp` | string (RFC3339) | イベント検知時刻（UTC） |

> **注意:** リネームは「削除 → 作成」が 100ms 以内に連続した場合のみ `file_renamed` として検出されます。それ以外は `file_deleted` と `file_created` の 2 イベントとして発火します。

#### 挙動の詳細

| 項目 | 仕様 |
|------|------|
| 起動前から存在するファイル | イベントを**発火しません**。起動後の変更のみ検知します。 |
| `--debounce` の動作 | パス単位で coalesce します。同一ファイルの連続変更は最後の 1 件にまとめます。 |
| `--filter` の適用範囲 | **ファイル名のみ**に glob を照合します。`src/**/*.rs` のようなパスを含むパターンは機能しません。`*.rs` と指定すると全ディレクトリの `.rs` ファイルにマッチします。 |
| `--ignore` の照合方法 | パスコンポーネント（ディレクトリ/ファイル名）との**完全一致**です。glob ではありません。`node_modules` と指定すると任意の深さの同名セグメントを除外します。 |
| `Access` イベント | 無視されます（読み取りアクセスではイベント発火しません）。 |
| 非 UTF-8 パス | `to_string_lossy()` で U+FFFD に置換されます。 |

#### サブスクリプションテンプレート例

```
変更されたファイルをコードレビューしてください: {{file_path}}
```

```
新しいファイルが追加されました。{{timestamp}} に作成された {{file_path}} を確認してください。
```

```
ファイルがリネームされました。
  変更前: {{old_path}}
  変更後: {{new_path}}
変更の意図を推測してコメントを更新してください。
```

---

### IMAP Watcher

IMAP メールボックスを監視し、新着メールが届いたときにイベントを発火します。

> **重要: 起動時の挙動について**
> プラグイン起動時点で既にメールボックスに存在する未読メールは「既知のメール」として扱われ、イベントを**発火しません**。起動後に新たに届いたメールのみが対象です。既存の未読メールを処理したい場合は、一度既読にしてから再度送信するなどの方法を使ってください。

#### コマンドライン引数

| 引数 | 説明 | デフォルト |
|------|------|-----------|
| `--server <host>` | IMAP サーバーホスト名（**必須**） | — |
| `--port <port>` | IMAP サーバーポート | `993` |
| `--user <username>` | ユーザー名（**必須**） | — |
| `--password <password>` | パスワード（**必須**） | — |
| `--folder <folder>` | 監視するフォルダ名 | `INBOX` |
| `--poll-interval <seconds>` | ポーリング間隔（秒）/ IDLE タイムアウト | `60` |
| `--tls` / `--no-tls` | TLS/SSL の使用 | TLS 有効 |
| `--subject-match <regex>` | 件名の正規表現フィルタ（省略可） | 全件 |
| `--body-match <regex>` | 本文の正規表現フィルタ（省略可） | 全件 |

> **セキュリティ注意:** `--password` の値は `config.json` に**平文**で保存されます。詳細は [#15](https://github.com/fuchigta/cc-launcher/issues/15) を参照してください。

#### イベントカタログ

##### `new_mail` — 新着メール

```json
{
  "eventType": "new_mail",
  "data": {
    "message_id": "<abc123@mail.example.com>",
    "from": "sender@example.com",
    "subject": "週次レポート",
    "date": "Mon, 1 Jan 2025 09:00:00 +0900",
    "body_text": "本文テキスト...",
    "body_html": "<html>...</html>",
    "timestamp": "2025-01-01T00:00:00.000Z"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `message_id` | string | メールの Message-ID ヘッダ |
| `from` | string | 送信者（From ヘッダ） |
| `subject` | string | 件名（Subject ヘッダ） |
| `date` | string | 送信日時（Date ヘッダ、RFC 2822 形式） |
| `body_text` | string | プレーンテキスト本文（`text/plain` パート） |
| `body_html` | string | HTML 本文（`text/html` パート） |
| `timestamp` | string (RFC3339) | cc-launcher がイベントを検知した時刻（UTC） |

> **注意:** `body_match` フィルタは `body_text`（text/plain）のみに適用されます。HTML のみのメールでは `body_match` が一致しない場合があります（[#17](https://github.com/fuchigta/cc-launcher/issues/17) 参照）。

#### 挙動の詳細

| 項目 | 仕様 |
|------|------|
| 検索クエリ | `UNSEEN`（未読）のみ。既読メールは検知されません。 |
| 既読フラグ | `BODY.PEEK[]` を使用するため、取得しても**既読にはなりません**。 |
| フィルタの大文字小文字 | `--subject-match` / `--body-match` は大文字小文字を**無視**します（自動で `(?i)` が付加）。 |
| 接続方式 | IDLE に対応しているサーバはリアルタイム通知を使用します。非対応の場合は `--poll-interval` 秒ごとにポーリングします。 |
| 再接続 | 切断時は指数バックオフ（最大 300 秒）で自動再接続します。 |

#### サブスクリプションテンプレート例

```
以下のメールを要約してください:

送信者: {{from}}
件名: {{subject}}
本文:
{{body_text}}
```

```
緊急メールが届きました。
送信者: {{from}}
件名: {{subject}}
受信日時: {{timestamp}}

このメールの要点を整理し、必要なアクションを提案してください。
```

---

## サブスクリプションテンプレート構文

### 基本構文: `{{key}}`

`{{key}}` 形式のプレースホルダが、イベントの `data` オブジェクトのトップレベルフィールドに置換されます。

```
変更ファイル: {{file_path}}  →  変更ファイル: C:\project\src\main.rs
```

### 置換ルール

| 状況 | 動作 |
|------|------|
| `data` にキーが存在する（文字列） | 値で置換 |
| `data` にキーが存在する（数値/bool/null） | `5` / `true` / `null` に文字列化して置換 |
| `data` にキーが存在する（オブジェクト/配列） | JSON 文字列（`{"id":1,...}` 等）に置換 |
| `data` にキーが存在しない | `{{key}}` のままプロンプトに残る |

### 制限事項

- **ネスト不可:** `{{user.name}}` のようなドット記法は**サポートしていません**。ネストされたオブジェクトは丸ごと JSON 文字列になります。
- **条件分岐・ループなし:** if/else や繰り返しの構文はありません。
- **マッチングは完全一致のみ:** `pluginName` / `eventType` は `*`（全マッチ）か完全一致のみです。プレフィックスやグロブは使えません。
- **エスケープなし:** `data` の値はそのままプロンプトに連結されます。監視ディレクトリへの書き込み権限を持つ第三者がファイル名を通じてプロンプトを操作できる可能性があります（[#15](https://github.com/fuchigta/cc-launcher/issues/15) 参照）。

### ワイルドカード

`Plugin Name` または `Event Type` に `*` を指定すると、それぞれ全プラグイン / 全イベント種別にマッチします。複数のサブスクリプションがマッチした場合は**全て並列に実行**されます（重複排除はされません）。

---

## カスタムプラグインの作成

任意の実行ファイルをプラグインとして登録できます。JSON-RPC 2.0 形式の stdin/stdout 通信を実装する必要があります。

Node.js のサンプル実装: [`examples/plugins/file-watcher/`](../examples/plugins/file-watcher/)

### プロトコル仕様

#### 1. initialize（必須）

cc-launcher が起動時に送信するリクエストです。**10 秒以内**にレスポンスを返す必要があります。

```json
// リクエスト (cc-launcher → plugin)
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"app_version":"0.22.0","plugin_name":"MyPlugin"}}

// レスポンス (plugin → cc-launcher)
{"jsonrpc":"2.0","id":1,"result":{"name":"my-plugin","version":"1.0.0"}}
```

#### 2. event 通知（任意）

プラグインが任意のタイミングで cc-launcher に送信する通知です。

```json
{"jsonrpc":"2.0","method":"event","params":{"eventType":"my_event","data":{"key":"value"}}}
```

- `eventType`: イベントの種別（サブスクリプションの `Event Type` フィールドと照合）
- `data`: 任意の JSON オブジェクト。キーがサブスクリプションテンプレートの `{{key}}` として使用可能

#### 3. shutdown（推奨）

cc-launcher がプラグイン停止時に送信するリクエストです。受信後に `std::process::exit(0)` するなどで正常終了してください。応答がない場合は 5 秒後に強制終了（SIGKILL）されます。

```json
// リクエスト (cc-launcher → plugin)
{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}
```

### 注意事項

- **stderr は破棄されます。** プラグインが stderr に出力してもcc-launcher では確認できません（[#16](https://github.com/fuchigta/cc-launcher/issues/16) 参照）。デバッグにはファイルへのログ出力を使ってください。
- JSON パースに失敗した行はサイレントに無視されます。
- cc-launcher のプラグインリストの **Name フィールド** がサブスクリプションの `Plugin Name` との照合キーになります。プラグイン名を変更すると既存サブスクリプションが動作しなくなります（[#16](https://github.com/fuchigta/cc-launcher/issues/16) 参照）。
