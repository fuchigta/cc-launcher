# playwright-cli によるUI確認手順

Tauriアプリはブラウザ単体ではTauri API（`invoke`, `getCurrentWindow`等）が存在しないためコンポーネントがエラーになる。`playwright-cli` の `addInitScript` でTauri APIモックを注入してからページを開くことで、ブラウザ上でもUIを確認できる。

## 前提

- `pnpm tauri dev` が起動済み（localhost:1420でViteが稼働）
- `playwright-cli` コマンドが利用可能

## 手順

### 1. ブラウザ起動

```bash
playwright-cli open "about:blank" --headed
```

### 2. Tauri APIモック注入 + ページ遷移

`addInitScript` でモックを登録してから `goto` する。これによりページ読み込み前にモックが設定される。

```bash
playwright-cli run-code "async (page) => { await page.addInitScript('<モックスクリプト>'); await page.goto('http://localhost:1420/#/'); }"
```

モックスクリプト（1行に圧縮したもの）:

```
window.__TAURI_INTERNALS__={invoke:function(cmd){var h={get_config:{shortcut:"Ctrl+Shift+Space",terminal:"Auto",wslShell:"Bash",lastDirectory:"C:\\project",recentDirectories:["C:\\project","C:\\other"],wslDirectory:null,wslRecentDirectories:[],schedules:[],plugins:[],subscriptions:[]},get_available_terminals:[{terminal_type:"Pwsh",display_name:"PowerShell 7",available:true},{terminal_type:"Cmd",display_name:"Command Prompt",available:true}],save_config:null,hide_window:null,open_claude_interactive:null,update_recent_directory:null,get_schedules:[],get_plugins:[],get_subscriptions:[],get_logs:[],get_plugin_statuses:[]};return Promise.resolve(h[cmd]!==undefined?h[cmd]:null)},metadata:{currentWindow:{label:"main"},currentWebview:{label:"main"}},transformCallback:function(cb,once){var id=Math.floor(Math.random()*100000);var p="_"+id;Object.defineProperty(window,p,{value:function(r){if(once)delete window[p];return cb&&cb(r)},writable:false,configurable:true});return id}}
```

### 3. 各ルートの確認

`addInitScript` は一度登録すれば以降の `goto` でも有効。

```bash
# メイン入力オーバーレイ
playwright-cli goto "http://localhost:1420/#/"
playwright-cli screenshot

# 設定画面
playwright-cli goto "http://localhost:1420/#/settings"
playwright-cli screenshot

# マネージャー画面
playwright-cli goto "http://localhost:1420/#/manager"
playwright-cli screenshot
```

### 4. 終了

```bash
playwright-cli close
```

## 確認ポイント

| ルート | 正常表示の目安 |
|--------|---------------|
| `/#/` | "Ask Claude..." placeholder、ディレクトリボタン |
| `/#/settings` | "Settings" 見出し、Global Shortcut入力、Terminal select、Save/Closeボタン |
| `/#/manager` | "Manager" 見出し、4タブ（Schedules, Plugins, Subscriptions, Logs） |

## 注意点

- モックの `invoke` はダミーデータを返すだけなので、保存操作等の副作用は発生しない
- 新しいTauriコマンドを追加した場合は、モックスクリプトの `h` オブジェクトにもキーを追加する必要がある
- `--headed` を外すとヘッドレスモードで動作する（CI向け）
- `playwright-cli eval` は関数を含むオブジェクトを渡せない制約がある。関数を含むモックは `run-code` + `addInitScript` を使う
