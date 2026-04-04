# Claude Code スキル

cc-launcher は [Claude Code](https://docs.anthropic.com/en/docs/claude-code) のプラグインとして、自然言語でcc-launcherを操作するスキルを提供します。

## 前提条件

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) がインストール済みであること
- cc-launcher がインストール済みで起動していること

## インストール

```bash
claude plugin add fuchigta/cc-launcher
```

---

## cc-launcher-cli スキル

Claude Codeのセッション内で自然言語によってcc-launcherのスケジュール・プラグイン・サブスクリプション・設定を操作できます。

**トリガー条件:** 以下のような発言をするとスキルが自動的に起動します。

- 「スケジュールを追加して」
- 「毎日○時にClaudeを動かして」
- 「プラグインを一覧して」
- 「cc-launcherの設定を変更して」
- 「サブスクリプションを追加して」

### 使用例

**スケジュール管理:**

```
毎朝9時に「今日のタスクリストを作って」というプロンプトを実行するスケジュールを追加して
```

```
スケジュール一覧を見せて
```

```
"朝のレポート"スケジュールを無効にして
```

**プラグイン管理:**

```
プラグインの一覧を表示して
```

```
C:/projects/myapp を監視するFolderWatcherプラグインを追加して
```

**サブスクリプション管理:**

```
ファイル変更イベントを受けたら「変更されたファイルをレビューして」と実行するサブスクリプションを追加して
```

**設定変更:**

```
ショートカットをCtrl+Alt+Cに変更して
```

```
実行タイムアウトを600秒に設定して
```

---

## 内部動作

スキルは裏側で `cc-launcher-cli` コマンドを呼び出します。CLIの詳細は [docs/cli.md](cli.md) を参照してください。設定はcc-launcherが管理する `%APPDATA%\cc-launcher\config.json` に即時反映され、実行中のcc-launcherにもホットリロードされます。
