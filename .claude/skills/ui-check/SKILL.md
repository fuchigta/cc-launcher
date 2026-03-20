---
name: ui-check
description: This skill should be used when UI changes are made, CSS is modified, components are updated, layout is changed, styles are added, visual changes are implemented, or the user asks to "UI確認", "画面確認", "レイアウト確認", "見た目を確認", "UIを検証", "UIの確認もして", "UIも確認". Runs playwright-cli to visually verify the changed screens using Tauri API mocks.
version: 0.1.0
---

# UI確認手順（playwright-cli）

## 概要

TauriアプリはブラウザだけではTauri APIが無いためエラーになる。`playwright-cli` の `addInitScript` でモックを注入してからページを開くことで、ブラウザ上でUIを確認する。

## 手順

### Step 1: `pnpm tauri dev` を起動

まず `localhost:1420` が応答しているか確認する。

```bash
curl -s http://localhost:1420 > /dev/null 2>&1 && echo "already running" || echo "not running"
```

応答しない場合はバックグラウンドで起動し、応答するまで待機する。

```bash
pnpm tauri dev 2>&1 &
for i in $(seq 1 30); do
  curl -s http://localhost:1420 > /dev/null 2>&1 && echo "Ready after ${i}s" && break
  sleep 2
done
```

### Step 2: playwright-cli 用スクリプトを一時ファイルに書く

**重要**: モック関数（`function`、`=>`等）はシェル文字列に埋め込むとエスケープが複雑になる。必ず `Write` ツールで一時ファイル（例: `C:\Users\<user>\AppData\Local\Temp\ui-check.js`）に書いてから `$(cat ...)` で渡すこと。

スクリプトのひな形:

```js
async (page) => {
  // 確認したい画面に応じてモックデータを用意する
  const mockData = { /* ... */ };

  await page.addInitScript((data) => {
    window.__TAURI_INTERNALS__ = {
      invoke: function (cmd) {
        const h = {
          get_config: {
            shortcut: "Ctrl+Shift+Space",
            terminal: "Auto",
            wslShell: "Bash",
            lastDirectory: "C:\\project",
            recentDirectories: ["C:\\project"],
            wslDirectory: null,
            wslRecentDirectories: [],
            schedules: [],
            plugins: [],
            subscriptions: [],
          },
          get_available_terminals: [
            { terminal_type: "Pwsh", display_name: "PowerShell 7", available: true },
          ],
          save_config: null,
          hide_window: null,
          open_claude_interactive: null,
          update_recent_directory: null,
          get_schedules: [],
          get_plugins: [],
          get_subscriptions: [],
          get_logs: [],
          get_plugin_statuses: [],
          // 確認対象のコマンドを追加する
        };
        return Promise.resolve(h[cmd] !== undefined ? h[cmd] : null);
      },
      metadata: {
        currentWindow: { label: "main" },
        currentWebview: { label: "main" },
      },
      transformCallback: function (cb, once) {
        const id = Math.floor(Math.random() * 100000);
        const p = "_" + id;
        Object.defineProperty(window, p, {
          value: function (r) {
            if (once) delete window[p];
            return cb && cb(r);
          },
          writable: false,
          configurable: true,
        });
        return id;
      },
    };
  }, mockData);

  // 変更した画面のルートへ遷移
  await page.goto("http://localhost:1420/#/<route>");
  await page.waitForTimeout(1000);

  // 必要に応じてタブクリックや行クリック等で状態を作る
  await page.screenshot({ path: "C:\\Users\\<user>\\AppData\\Local\\Temp\\ui-check-01.png" });

  // 追加の操作と検証...
}
```

#### ルート一覧

| 画面 | ルート |
|------|--------|
| メイン入力オーバーレイ | `/#/` |
| 設定画面 | `/#/settings` |
| マネージャー | `/#/manager` |

#### 新しいTauriコマンドを追加した場合

`invoke` の `h` オブジェクトに対応するキーを追加する。戻り値が不要なコマンドは `null`。

### Step 3: スクリプトを実行

```bash
playwright-cli run-code "$(cat 'C:\Users\<user>\AppData\Local\Temp\ui-check.js')" 2>&1
```

### Step 4: スクリーンショットを確認

`Read` ツールでスクリーンショットを開き、変更内容が意図通りに表示されているかを目視確認する。

- レイアウトの崩れがないか
- ボタン・バッジ等の要素が正しい位置にあるか
- `sessionId` の有無など条件分岐が正しく動作しているか

## 注意事項

- モックは副作用を起こさないので保存・送信等の操作は実際には反映されない
- `durationMs` は `bigint | null` 型だが、モックでは数値リテラルで渡して問題ない
- `playwright-cli eval` は関数を含むオブジェクトを渡せない → `run-code` + `addInitScript` + 一時ファイルを使う
- `--headed` を付けると有頭モード（ウィンドウが開く）で実行できる
