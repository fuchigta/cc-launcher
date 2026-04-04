# cc-launcher-cli リファレンス

`cc-launcher-cli` はcc-launcherのスケジュール・プラグイン・サブスクリプション・設定をコマンドラインから管理するツールです。

## 基本構文

```
cc-launcher-cli [--json] <COMMAND>
```

| オプション | 説明 |
|-----------|------|
| `--json` | 出力をJSON形式にする（スクリプト連携に有用） |

サブコマンドは `schedule` / `plugin` / `subscription` / `config` の4種類です。

---

## schedule

スケジュールの一覧・追加・削除・有効化・無効化。

### 一覧

```bash
cc-launcher-cli schedule list
cc-launcher-cli --json schedule list
```

### 追加

スケジュール表現は `--cron` / `--interval` / `--daily-at` のいずれか1つを必ず指定します。

```bash
# cron式で毎日9時に実行
cc-launcher-cli schedule add \
  --name "朝のレポート" \
  --cron "0 9 * * *" \
  --prompt "今日のタスクをまとめて" \
  --dir "C:/Users/me/projects/myapp"

# 30分ごとに実行
cc-launcher-cli schedule add \
  --name "定期チェック" \
  --interval 1800 \
  --prompt "ログを確認してエラーを報告して"

# 毎日22:00に実行
cc-launcher-cli schedule add \
  --name "夜間レポート" \
  --daily-at "22:00" \
  --prompt "今日の作業サマリーを作成して"

# プロンプトをstdinから読み込む
echo "複雑なプロンプトをここに" | cc-launcher-cli schedule add \
  --name "複雑なタスク" \
  --daily-at "08:00" \
  --prompt -

# claude引数を追加する場合
cc-launcher-cli schedule add \
  --name "詳細モード" \
  --cron "0 10 * * 1" \
  --prompt "週次レポートを作成して" \
  --arg "--model=claude-opus-4-6"
```

**オプション:**

| オプション | 説明 |
|-----------|------|
| `--name` | スケジュール名（必須） |
| `--cron <式>` | cron式（例: `"0 9 * * *"` = 毎日9時） |
| `--interval <秒>` | インターバル秒数（例: `3600` = 1時間ごと） |
| `--daily-at <HH:MM>` | 毎日の実行時刻（例: `"09:30"`） |
| `--prompt <テキスト>` | プロンプトテキスト（`-` でstdinから読み込み） |
| `--dir <パス>` | 作業ディレクトリ（省略可） |
| `--arg <値>` | claude追加引数（複数回指定可） |

### 削除・有効化・無効化

```bash
cc-launcher-cli schedule delete "朝のレポート"
cc-launcher-cli schedule delete abc12345
cc-launcher-cli schedule disable "朝のレポート"
cc-launcher-cli schedule enable abc12345
```

---

## plugin

プラグインの一覧・追加・削除・有効化・無効化。

### 一覧

```bash
cc-launcher-cli plugin list
cc-launcher-cli --json plugin list
```

### 追加

```bash
# 外部スクリプトをプラグインとして追加
cc-launcher-cli plugin add \
  --name "ファイル監視" \
  --executable "C:/tools/file-watcher.exe"

# 引数付きで追加
cc-launcher-cli plugin add \
  --name "Gitモニター" \
  --executable "python" \
  --arg "C:/tools/git-monitor.py" \
  --arg "--watch-dir=C:/projects"
```

**オプション:**

| オプション | 説明 |
|-----------|------|
| `--name` | プラグイン名（必須） |
| `--executable <パス>` | 実行ファイルパス（必須） |
| `--arg <値>` | 引数（複数回指定可） |

> **注意:** `Folder Watcher` や `IMAP Watcher` などのビルトインプラグインはGUIのManagerから設定してください。

### 削除・有効化・無効化

```bash
cc-launcher-cli plugin delete "ファイル監視"
cc-launcher-cli plugin disable "Gitモニター"
cc-launcher-cli plugin enable abc12345
```

---

## subscription

サブスクリプション（プラグインイベントへの購読）の一覧・追加・削除・有効化・無効化。

### 一覧

```bash
cc-launcher-cli subscription list
cc-launcher-cli --json subscription list
```

### 追加

```bash
# プラグインのイベントを購読してClaudeに処理させる
cc-launcher-cli subscription add \
  --name "エラー検知通知" \
  --plugin "Gitモニター" \
  --event "error_detected" \
  --template "以下のエラーを分析して修正案を提示して:\n{{event_data}}"

# テンプレートをstdinから読み込む
cat prompt-template.txt | cc-launcher-cli subscription add \
  --name "複雑な処理" \
  --plugin "ファイル監視" \
  --event "file_changed" \
  --template -

# 作業ディレクトリ・引数指定
cc-launcher-cli subscription add \
  --name "ビルドエラー対応" \
  --plugin "CIモニター" \
  --event "build_failed" \
  --template "ビルドエラーを修正して: {{message}}" \
  --dir "C:/projects/myapp" \
  --arg "--model=claude-opus-4-6"
```

**オプション:**

| オプション | 説明 |
|-----------|------|
| `--name` | サブスクリプション名（必須） |
| `--plugin <名前>` | 購読するプラグイン名（必須） |
| `--event <種別>` | イベント種別（必須） |
| `--template <テキスト>` | プロンプトテンプレート（`-` でstdinから読み込み） |
| `--dir <パス>` | 作業ディレクトリ（省略可） |
| `--arg <値>` | claude追加引数（複数回指定可） |

### 削除・有効化・無効化

```bash
cc-launcher-cli subscription delete "エラー検知通知"
cc-launcher-cli subscription disable abc12345
cc-launcher-cli subscription enable "ビルドエラー対応"
```

---

## config

設定の表示・変更。

### 設定表示

```bash
cc-launcher-cli config show
cc-launcher-cli --json config show
```

### 設定変更

```bash
# グローバルキーボードショートカット変更
cc-launcher-cli config set shortcut "Ctrl+Shift+Space"
cc-launcher-cli config set shortcut "Alt+F1"

# 実行タイムアウト変更（秒）
cc-launcher-cli config set timeout 300

# ターミナル種別変更
cc-launcher-cli config set terminal Auto       # 自動選択
cc-launcher-cli config set terminal Pwsh       # PowerShell 7+
cc-launcher-cli config set terminal PowerShell # Windows PowerShell
cc-launcher-cli config set terminal Cmd        # コマンドプロンプト
cc-launcher-cli config set terminal Wsl        # WSL
```

---

## ID解決ルール

`delete` / `enable` / `disable` では以下の順でIDを解決します:

1. **UUID完全一致** — `550e8400-e29b-41d4-a716-446655440000`
2. **名前完全一致** — `朝のレポート`
3. **UUIDプリフィックス** — `550e8400`（先頭数文字）

プリフィックスが複数件にマッチした場合はエラーになります。

---

## JSON出力の活用

```bash
# 追加したスケジュールのIDを取得
ID=$(cc-launcher-cli --json schedule add --name "test" --daily-at "12:00" --prompt "hello" | jq -r .id)

# 一覧をjqでフィルタ
cc-launcher-cli --json schedule list | jq '.[] | select(.enabled == true)'
cc-launcher-cli --json plugin list | jq '.[].name'
```

---

## stdin入力の活用

`--prompt -` または `--template -` を指定するとstdinからテキストを読み込みます。

```bash
# ファイルからプロンプトを読み込む
cat my-prompt.txt | cc-launcher-cli schedule add \
  --name "長いプロンプト" \
  --daily-at "09:00" \
  --prompt -

# ヒアドキュメント
cc-launcher-cli subscription add \
  --name "詳細分析" \
  --plugin "monitor" \
  --event "alert" \
  --template - << 'EOF'
以下のアラートを分析してください:
{{event_data}}

重要度を判定し、必要なら修正案を提示してください。
EOF
```
