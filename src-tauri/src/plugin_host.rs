use crate::models::{
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, PluginConfig, PluginEvent, PluginStatus,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, RwLock};

struct PluginHandle {
    config: PluginConfig,
    child: Option<Child>,
    running: bool,
    pid: Option<u32>,
    last_event_at: Option<chrono::DateTime<Utc>>,
    error: Option<String>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, PluginHandle>>>,
    event_tx: mpsc::Sender<(String, PluginEvent)>,
    app_handle: tauri::AppHandle,
}

impl PluginManager {
    pub fn new(
        event_tx: mpsc::Sender<(String, PluginEvent)>,
        app_handle: tauri::AppHandle,
    ) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            app_handle,
        }
    }

    pub async fn start_plugin(&self, config: &PluginConfig) -> Result<(), String> {
        self.stop_plugin(&config.id).await.ok();

        let mut std_cmd = std::process::Command::new(&config.executable);
        for arg in &config.args {
            std_cmd.arg(arg);
        }

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        use std::os::windows::process::CommandExt;
        std_cmd.creation_flags(CREATE_NO_WINDOW);
        std_cmd.stdin(std::process::Stdio::piped());
        std_cmd.stdout(std::process::Stdio::piped());
        std_cmd.stderr(std::process::Stdio::null());

        let mut cmd = Command::from(std_cmd);
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn plugin {}: {}", config.name, e))?;

        let pid = child.id();

        // Send initialize request
        let stdin = child.stdin.as_mut().ok_or("Failed to get stdin")?;
        let init_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(1.into()),
            method: "initialize".to_string(),
            params: serde_json::json!({
                "app_version": env!("CARGO_PKG_VERSION"),
                "plugin_name": config.name,
            }),
        };
        let mut msg = serde_json::to_string(&init_request).map_err(|e| e.to_string())?;
        msg.push('\n');
        stdin
            .write_all(msg.as_bytes())
            .await
            .map_err(|e| format!("Failed to write init: {}", e))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush: {}", e))?;

        // Read initialize response (with timeout)
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        let read_result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            reader.read_line(&mut line),
        )
        .await;

        match read_result {
            Ok(Ok(0)) => return Err("Plugin closed stdout before responding".to_string()),
            Ok(Ok(_)) => {
                let _response: JsonRpcResponse = serde_json::from_str(line.trim())
                    .map_err(|e| format!("Invalid init response: {}", e))?;
            }
            Ok(Err(e)) => return Err(format!("Failed to read init response: {}", e)),
            Err(_) => return Err("Plugin init timed out".to_string()),
        }

        // Spawn stdout reader task
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        let event_tx = self.event_tx.clone();
        let plugin_name = config.name.clone();
        let plugin_id = config.id.clone();
        let plugins = self.plugins.clone();
        let app_handle = self.app_handle.clone();
        let plugin_config = config.clone();

        tokio::spawn(async move {
            loop {
                let mut buf = String::new();
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        break;
                    }
                    result = reader.read_line(&mut buf) => {
                        match result {
                            Ok(0) | Err(_) => {
                                let mut handles = plugins.write().await;
                                if let Some(handle) = handles.get_mut(&plugin_id) {
                                    handle.running = false;
                                    handle.error = Some("Process exited".to_string());
                                }
                                drop(handles);
                                let _ = app_handle.emit("plugin-status-changed", &plugin_id);
                                eprintln!("Plugin {} exited", plugin_config.name);
                                break;
                            }
                            Ok(_) => {
                                let trimmed = buf.trim();
                                if trimmed.is_empty() {
                                    continue;
                                }
                                let notification = match serde_json::from_str::<JsonRpcNotification>(trimmed) {
                                    Ok(n) if n.method == "event" => n,
                                    _ => continue,
                                };
                                let event = match serde_json::from_value::<PluginEvent>(notification.params) {
                                    Ok(e) => e,
                                    Err(_) => continue,
                                };
                                let mut handles = plugins.write().await;
                                if let Some(handle) = handles.get_mut(&plugin_id) {
                                    handle.last_event_at = Some(Utc::now());
                                }
                                drop(handles);
                                let _ = event_tx.send((plugin_name.clone(), event)).await;
                            }
                        }
                    }
                }
            }
        });

        let handle = PluginHandle {
            config: config.clone(),
            child: Some(child),
            running: true,
            pid,
            last_event_at: None,
            error: None,
            shutdown_tx: Some(shutdown_tx),
        };

        let mut handles = self.plugins.write().await;
        handles.insert(config.id.clone(), handle);

        let _ = self.app_handle.emit("plugin-status-changed", &config.id);

        Ok(())
    }

    pub async fn stop_plugin(&self, id: &str) -> Result<(), String> {
        let mut handles = self.plugins.write().await;
        if let Some(handle) = handles.get_mut(id) {
            // Send shutdown via stdin if process is running
            if let Some(child) = handle.child.as_mut() {
                if let Some(stdin) = child.stdin.as_mut() {
                    let shutdown_req = JsonRpcRequest {
                        jsonrpc: "2.0".to_string(),
                        id: serde_json::Value::Number(99.into()),
                        method: "shutdown".to_string(),
                        params: serde_json::Value::Null,
                    };
                    let mut msg = serde_json::to_string(&shutdown_req).unwrap_or_default();
                    msg.push('\n');
                    let _ = stdin.write_all(msg.as_bytes()).await;
                    let _ = stdin.flush().await;
                }
            }

            // Signal the reader task to stop
            if let Some(tx) = handle.shutdown_tx.take() {
                let _ = tx.send(());
            }

            // Wait briefly then kill
            if let Some(mut child) = handle.child.take() {
                let kill_result =
                    tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await;

                if kill_result.is_err() {
                    let _ = child.kill().await;
                }
            }

            handle.running = false;
            handle.pid = None;
        }

        let _ = self.app_handle.emit("plugin-status-changed", id);
        Ok(())
    }

    pub async fn get_statuses(&self) -> Vec<PluginStatus> {
        let handles = self.plugins.read().await;
        handles
            .values()
            .map(|h| PluginStatus {
                id: h.config.id.clone(),
                name: h.config.name.clone(),
                running: h.running,
                pid: h.pid,
                last_event_at: h.last_event_at,
                error: h.error.clone(),
            })
            .collect()
    }

    pub async fn start_all(&self, configs: &[PluginConfig]) {
        for config in configs {
            if config.enabled {
                if let Err(e) = self.start_plugin(config).await {
                    eprintln!("Failed to start plugin {}: {}", config.name, e);
                }
            }
        }
    }

    pub async fn stop_all(&self) {
        let ids: Vec<String> = {
            let handles = self.plugins.read().await;
            handles.keys().cloned().collect()
        };
        for id in ids {
            let _ = self.stop_plugin(&id).await;
        }
    }
}
