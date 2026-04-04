---
paths:
  - "src-tauri/src/models.rs"
  - "src-tauri/src/config.rs"
---

## Rust型定義の変更時ルール

### ts-rs derive
フロントエンドと共有する型には必ず以下を付与すること:

```rust
#[derive(ts_rs::TS)]
#[ts(export)]
```

`serde` の rename 属性がある場合は `ts` にも対応する属性を追加すること:

| serde | ts |
|-------|----|
| `#[serde(rename_all = "camelCase")]` | `#[ts(rename_all = "camelCase")]` |
| `#[serde(rename = "foo")]` | `#[ts(rename = "foo")]` |

### AppConfig フィールド追加時
`config.rs` の `AppConfig` にフィールドを追加した場合は、`src/Settings.tsx` の `handleSave` で新フィールドを必ず pass through すること。
