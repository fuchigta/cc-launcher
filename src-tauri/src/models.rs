#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Execution ---

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ExecutionSource {
    Schedule {
        id: String,
        name: String,
    },
    Plugin {
        #[serde(rename = "pluginName")]
        #[ts(rename = "pluginName")]
        plugin_name: String,
        #[serde(rename = "eventType")]
        #[ts(rename = "eventType")]
        event_type: String,
    },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ts_rs::TS)]
#[ts(export)]
pub enum ExecutionStatus {
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ExecutionLog {
    pub id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    pub source: ExecutionSource,
    pub prompt: String,
    pub working_dir: Option<String>,
    pub claude_args: Vec<String>,
    pub status: ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

// --- Schedule ---

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ScheduleExpression {
    Cron { expression: String },
    Interval { seconds: u64 },
    DailyAt { time: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub id: String,
    pub name: String,
    pub expression: ScheduleExpression,
    pub prompt: String,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub claude_args: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

// --- Plugin ---

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PluginConfig {
    pub id: String,
    pub name: String,
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PluginStatus {
    pub id: String,
    pub name: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub last_event_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

// --- JSON-RPC 2.0 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PluginEvent {
    pub event_type: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

// --- Subscription ---

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SubscriptionConfig {
    pub id: String,
    pub name: String,
    pub plugin_name: String,
    pub event_type: String,
    pub prompt_template: String,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub claude_args: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_source_schedule_serde() {
        let source = ExecutionSource::Schedule {
            id: "s1".to_string(),
            name: "daily".to_string(),
        };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"type\":\"Schedule\""));
        assert!(json.contains("\"id\":\"s1\""));

        let restored: ExecutionSource = serde_json::from_str(&json).unwrap();
        match restored {
            ExecutionSource::Schedule { id, name } => {
                assert_eq!(id, "s1");
                assert_eq!(name, "daily");
            }
            _ => panic!("Expected Schedule variant"),
        }
    }

    #[test]
    fn execution_source_manual_serde() {
        let source = ExecutionSource::Manual;
        let json = serde_json::to_string(&source).unwrap();
        let restored: ExecutionSource = serde_json::from_str(&json).unwrap();
        assert!(matches!(restored, ExecutionSource::Manual));
    }

    #[test]
    fn schedule_expression_variants() {
        let cron = ScheduleExpression::Cron {
            expression: "0 * * * *".to_string(),
        };
        let json = serde_json::to_string(&cron).unwrap();
        assert!(json.contains("\"type\":\"Cron\""));

        let interval = ScheduleExpression::Interval { seconds: 300 };
        let json = serde_json::to_string(&interval).unwrap();
        assert!(json.contains("\"seconds\":300"));

        let daily = ScheduleExpression::DailyAt {
            time: "09:00".to_string(),
        };
        let json = serde_json::to_string(&daily).unwrap();
        assert!(json.contains("\"time\":\"09:00\""));
    }

    #[test]
    fn plugin_event_serde() {
        let json = r#"{"eventType":"file_changed","data":{"path":"/tmp/test.txt"}}"#;
        let event: PluginEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "file_changed");
        assert_eq!(event.data["path"], "/tmp/test.txt");
    }

    #[test]
    fn schedule_config_defaults() {
        let json = r#"{
            "id": "s1",
            "name": "test",
            "expression": {"type": "Interval", "seconds": 60},
            "prompt": "hello",
            "workingDir": null
        }"#;
        let config: ScheduleConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert!(config.claude_args.is_empty());
    }
}
