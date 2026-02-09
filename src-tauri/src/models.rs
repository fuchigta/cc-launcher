#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Execution ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExecutionSource {
    Schedule {
        id: String,
        name: String,
    },
    Plugin {
        #[serde(rename = "pluginName")]
        plugin_name: String,
        #[serde(rename = "eventType")]
        event_type: String,
    },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLog {
    pub id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScheduleExpression {
    Cron { expression: String },
    Interval { seconds: u64 },
    DailyAt { time: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfig {
    pub id: String,
    pub name: String,
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginEvent {
    pub event_type: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

// --- Subscription ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
