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
    Cancelled,
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
    pub last_stderr: Option<String>,
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
    fn execution_status_cancelled_serde() {
        let status = ExecutionStatus::Cancelled;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Cancelled\"");
        let restored: ExecutionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, ExecutionStatus::Cancelled);
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

    mod props {
        use super::super::*;
        use proptest::collection::{btree_map, vec};
        use proptest::option;
        use proptest::prelude::*;
        use proptest::test_runner::TestCaseError;
        use serde::de::DeserializeOwned;
        use serde_json::Value;

        /// serialize -> deserialize -> serialize must reproduce the first JSON.
        /// Compares JSON text, so types without `PartialEq` can be checked too.
        fn assert_json_roundtrip<T>(value: &T) -> Result<(), TestCaseError>
        where
            T: Serialize + DeserializeOwned,
        {
            let first = serde_json::to_string(value).unwrap();
            let restored: T = serde_json::from_str(&first).unwrap();
            prop_assert_eq!(&first, &serde_json::to_string(&restored).unwrap());
            Ok(())
        }

        /// Arbitrary JSON without floats: serde_json is built without the
        /// `float_roundtrip` feature, so some `f64` values are not guaranteed to
        /// parse back bit-for-bit. That is a serde_json trade-off, not part of
        /// these models' contract, so floats are left out of the strategy.
        fn json_value() -> impl Strategy<Value = Value> {
            let leaf = prop_oneof![
                Just(Value::Null),
                any::<bool>().prop_map(Value::from),
                any::<i64>().prop_map(Value::from),
                any::<u64>().prop_map(Value::from),
                any::<String>().prop_map(Value::from),
            ];
            leaf.prop_recursive(3, 16, 4, |inner| {
                prop_oneof![
                    vec(inner.clone(), 0..4).prop_map(Value::Array),
                    btree_map(any::<String>(), inner, 0..4)
                        .prop_map(|m| Value::Object(m.into_iter().collect())),
                ]
            })
        }

        /// `Option<Value>` fields with `skip_serializing_if = "Option::is_none"`:
        /// `Some(Value::Null)` serializes as `null`, which serde reads back as
        /// `None`, so the key disappears on re-serialization. This is a known
        /// asymmetry of `Option<serde_json::Value>` (the host currently only
        /// parses `JsonRpcResponse` and never re-serializes it), so the strategy
        /// generates `None` or a non-null value only.
        fn opt_non_null_json() -> impl Strategy<Value = Option<Value>> {
            option::of(json_value().prop_filter("non-null", |v| !v.is_null()))
        }

        fn opt_string() -> impl Strategy<Value = Option<String>> {
            option::of(any::<String>())
        }

        fn strings() -> impl Strategy<Value = Vec<String>> {
            vec(any::<String>(), 0..4)
        }

        fn execution_source() -> impl Strategy<Value = ExecutionSource> {
            prop_oneof![
                (any::<String>(), any::<String>())
                    .prop_map(|(id, name)| ExecutionSource::Schedule { id, name }),
                (any::<String>(), any::<String>()).prop_map(|(plugin_name, event_type)| {
                    ExecutionSource::Plugin {
                        plugin_name,
                        event_type,
                    }
                }),
                Just(ExecutionSource::Manual),
            ]
        }

        fn execution_status() -> impl Strategy<Value = ExecutionStatus> {
            prop_oneof![
                Just(ExecutionStatus::Running),
                Just(ExecutionStatus::Success),
                Just(ExecutionStatus::Failed),
                Just(ExecutionStatus::Cancelled),
            ]
        }

        fn schedule_expression() -> impl Strategy<Value = ScheduleExpression> {
            prop_oneof![
                any::<String>().prop_map(|expression| ScheduleExpression::Cron { expression }),
                any::<u64>().prop_map(|seconds| ScheduleExpression::Interval { seconds }),
                any::<String>().prop_map(|time| ScheduleExpression::DailyAt { time }),
            ]
        }

        fn schedule_config() -> impl Strategy<Value = ScheduleConfig> {
            (
                any::<String>(),
                any::<String>(),
                schedule_expression(),
                any::<String>(),
                opt_string(),
                strings(),
                any::<bool>(),
            )
                .prop_map(
                    |(id, name, expression, prompt, working_dir, claude_args, enabled)| {
                        ScheduleConfig {
                            id,
                            name,
                            expression,
                            prompt,
                            working_dir,
                            claude_args,
                            enabled,
                        }
                    },
                )
        }

        fn plugin_config() -> impl Strategy<Value = PluginConfig> {
            (
                any::<String>(),
                any::<String>(),
                any::<String>(),
                strings(),
                any::<bool>(),
            )
                .prop_map(|(id, name, executable, args, enabled)| PluginConfig {
                    id,
                    name,
                    executable,
                    args,
                    enabled,
                })
        }

        fn subscription_config() -> impl Strategy<Value = SubscriptionConfig> {
            (
                (any::<String>(), any::<String>()),
                (any::<String>(), any::<String>()),
                any::<String>(),
                opt_string(),
                strings(),
                any::<bool>(),
            )
                .prop_map(
                    |(
                        (id, name),
                        (plugin_name, event_type),
                        prompt_template,
                        wd,
                        args,
                        enabled,
                    )| {
                        SubscriptionConfig {
                            id,
                            name,
                            plugin_name,
                            event_type,
                            prompt_template,
                            working_dir: wd,
                            claude_args: args,
                            enabled,
                        }
                    },
                )
        }

        fn plugin_event() -> impl Strategy<Value = PluginEvent> {
            (any::<String>(), json_value())
                .prop_map(|(event_type, data)| PluginEvent { event_type, data })
        }

        fn json_rpc_request() -> impl Strategy<Value = JsonRpcRequest> {
            (any::<String>(), json_value(), any::<String>(), json_value()).prop_map(
                |(jsonrpc, id, method, params)| JsonRpcRequest {
                    jsonrpc,
                    id,
                    method,
                    params,
                },
            )
        }

        fn json_rpc_error() -> impl Strategy<Value = JsonRpcError> {
            (any::<i64>(), any::<String>(), opt_non_null_json()).prop_map(
                |(code, message, data)| JsonRpcError {
                    code,
                    message,
                    data,
                },
            )
        }

        fn json_rpc_response() -> impl Strategy<Value = JsonRpcResponse> {
            (
                any::<String>(),
                json_value(),
                opt_non_null_json(),
                option::of(json_rpc_error()),
            )
                .prop_map(|(jsonrpc, id, result, error)| JsonRpcResponse {
                    jsonrpc,
                    id,
                    result,
                    error,
                })
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(64))]

            #[test]
            fn execution_source_roundtrip(v in execution_source()) {
                assert_json_roundtrip(&v)?;
            }

            #[test]
            fn execution_status_roundtrip(v in execution_status()) {
                let json = serde_json::to_string(&v).unwrap();
                prop_assert_eq!(serde_json::from_str::<ExecutionStatus>(&json).unwrap(), v);
            }

            #[test]
            fn schedule_expression_roundtrip(v in schedule_expression()) {
                assert_json_roundtrip(&v)?;
            }

            #[test]
            fn schedule_config_roundtrip(v in schedule_config()) {
                assert_json_roundtrip(&v)?;
            }

            #[test]
            fn plugin_config_roundtrip(v in plugin_config()) {
                assert_json_roundtrip(&v)?;
            }

            #[test]
            fn subscription_config_roundtrip(v in subscription_config()) {
                assert_json_roundtrip(&v)?;
            }

            #[test]
            fn plugin_event_roundtrip(v in plugin_event()) {
                assert_json_roundtrip(&v)?;
            }

            #[test]
            fn json_rpc_request_roundtrip(v in json_rpc_request()) {
                assert_json_roundtrip(&v)?;
            }

            #[test]
            fn json_rpc_response_roundtrip(v in json_rpc_response()) {
                assert_json_roundtrip(&v)?;
            }
        }
    }
}
