use crate::models::{
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, PluginConfig, PluginEvent, PluginStatus,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use tokio::sync::{mpsc, RwLock};

type PluginMap = Arc<RwLock<HashMap<String, PluginHandle>>>;

struct PluginHandle {
    config: PluginConfig,
    child: Option<Child>,
    running: bool,
    pid: Option<u32>,
    last_event_at: Option<chrono::DateTime<Utc>>,
    error: Option<String>,
    last_stderr: Option<String>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, PluginHandle>>>,
    event_tx: mpsc::Sender<(String, String, PluginEvent)>,
    app_handle: tauri::AppHandle,
}

impl PluginManager {
    pub fn new(
        event_tx: mpsc::Sender<(String, String, PluginEvent)>,
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

        let exe_path = self.resolve_executable(config)?;

        let mut child = spawn_plugin_process(config, &exe_path)?;

        let pid = child.id();

        // Send initialize request
        send_initialize_request(&mut child, config).await?;

        // Read initialize response (with timeout)
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let mut reader = BufReader::new(stdout);
        read_initialize_response(&mut reader).await?;

        // Spawn stderr reader task
        let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
        tokio::spawn(read_stderr_lines(
            self.plugins.clone(),
            config.id.clone(),
            BufReader::new(stderr),
        ));

        // Spawn stdout reader task
        let shutdown_tx = self.spawn_stdout_reader(config, reader);

        let handle = PluginHandle {
            config: config.clone(),
            child: Some(child),
            running: true,
            pid,
            last_event_at: None,
            error: None,
            last_stderr: None,
            shutdown_tx: Some(shutdown_tx),
        };

        let mut handles = self.plugins.write().await;
        handles.insert(config.id.clone(), handle);

        let _ = self.app_handle.emit("plugin-status-changed", &config.id);

        Ok(())
    }

    /// Resolve the executable path, expanding `sidecar:<name>` to the bundled binary.
    fn resolve_executable(&self, config: &PluginConfig) -> Result<String, String> {
        if config.executable.starts_with("sidecar:") {
            let sidecar_name = &config.executable["sidecar:".len()..];
            self.resolve_sidecar_path(sidecar_name)
        } else {
            Ok(config.executable.clone())
        }
    }

    fn resolve_sidecar_path(&self, sidecar_name: &str) -> Result<String, String> {
        let bin_name = format!("plugins/{sidecar_name}{}", std::env::consts::EXE_SUFFIX);

        // Try resource_dir first (production build)
        let resource_path = self
            .app_handle
            .path()
            .resource_dir()
            .ok()
            .map(|d| d.join(&bin_name));

        let Some(ref p) = resource_path else {
            return Err(format!(
                "Failed to resolve resource dir for sidecar: {}",
                sidecar_name
            ));
        };
        if p.exists() {
            return Ok(p.to_string_lossy().to_string());
        }

        // Fallback: look in cargo target directory (dev mode)
        let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/debug")
            .join(format!("{sidecar_name}{}", std::env::consts::EXE_SUFFIX));
        if dev_path.exists() {
            Ok(dev_path.to_string_lossy().to_string())
        } else {
            Err(format!(
                "Sidecar binary not found: {} (tried {} and {})",
                sidecar_name,
                p.display(),
                dev_path.display()
            ))
        }
    }

    /// Spawn the stdout reader task and return the sender that stops it.
    fn spawn_stdout_reader(
        &self,
        config: &PluginConfig,
        reader: BufReader<ChildStdout>,
    ) -> tokio::sync::oneshot::Sender<()> {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let event_tx = self.event_tx.clone();
        let plugin_name = config.name.clone();
        let plugin_id = config.id.clone();
        let plugins = self.plugins.clone();
        let app_handle = self.app_handle.clone();

        tokio::spawn(read_stdout_events(
            reader,
            shutdown_rx,
            event_tx,
            plugin_id,
            plugin_name,
            plugins,
            app_handle,
        ));

        shutdown_tx
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
                last_stderr: h.last_stderr.clone(),
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

fn spawn_plugin_process(config: &PluginConfig, exe_path: &str) -> Result<Child, String> {
    let mut std_cmd = crate::windows_util::no_window_command(exe_path);
    for arg in &config.args {
        std_cmd.arg(arg);
    }

    std_cmd.stdin(std::process::Stdio::piped());
    std_cmd.stdout(std::process::Stdio::piped());
    std_cmd.stderr(std::process::Stdio::piped());

    let mut cmd = Command::from(std_cmd);
    cmd.spawn()
        .map_err(|e| format!("Failed to spawn plugin {}: {}", config.name, e))
}

async fn send_initialize_request(child: &mut Child, config: &PluginConfig) -> Result<(), String> {
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
    Ok(())
}

async fn read_initialize_response(reader: &mut BufReader<ChildStdout>) -> Result<(), String> {
    let mut line = String::new();

    let read_result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        reader.read_line(&mut line),
    )
    .await;

    match read_result {
        Ok(Ok(0)) => Err("Plugin closed stdout before responding".to_string()),
        Ok(Ok(_)) => {
            let _response: JsonRpcResponse = serde_json::from_str(line.trim())
                .map_err(|e| format!("Invalid init response: {}", e))?;
            Ok(())
        }
        Ok(Err(e)) => Err(format!("Failed to read init response: {}", e)),
        Err(_) => Err("Plugin init timed out".to_string()),
    }
}

/// Keep the latest non-empty stderr line of the plugin in its handle.
async fn read_stderr_lines(
    plugins: PluginMap,
    plugin_id: String,
    mut stderr_reader: BufReader<ChildStderr>,
) {
    let mut buf = String::new();
    loop {
        buf.clear();
        match stderr_reader.read_line(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(_) => record_stderr_line(&plugins, &plugin_id, buf.trim().to_string()).await,
        }
    }
}

async fn record_stderr_line(plugins: &PluginMap, plugin_id: &str, line: String) {
    if !line.is_empty() {
        let mut handles = plugins.write().await;
        if let Some(handle) = handles.get_mut(plugin_id) {
            handle.last_stderr = Some(line);
        }
    }
}

/// Forward plugin events from stdout until shutdown is signaled or the process exits.
async fn read_stdout_events(
    mut reader: BufReader<ChildStdout>,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    event_tx: mpsc::Sender<(String, String, PluginEvent)>,
    plugin_id: String,
    plugin_name: String,
    plugins: PluginMap,
    app_handle: tauri::AppHandle,
) {
    loop {
        let mut buf = String::new();
        tokio::select! {
            _ = &mut shutdown_rx => {
                break;
            }
            result = reader.read_line(&mut buf) => {
                match result {
                    Ok(0) | Err(_) => {
                        mark_plugin_exited(&plugins, &plugin_id, &plugin_name, &app_handle).await;
                        break;
                    }
                    Ok(_) => {
                        handle_stdout_line(buf.trim(), &plugins, &plugin_id, &plugin_name, &event_tx).await;
                    }
                }
            }
        }
    }
}

async fn mark_plugin_exited(
    plugins: &PluginMap,
    plugin_id: &str,
    plugin_name: &str,
    app_handle: &tauri::AppHandle,
) {
    let mut handles = plugins.write().await;
    if let Some(handle) = handles.get_mut(plugin_id) {
        handle.running = false;
        handle.error = Some("Process exited".to_string());
    }
    drop(handles);
    let _ = app_handle.emit("plugin-status-changed", plugin_id);
    eprintln!("Plugin {} exited", plugin_name);
}

async fn handle_stdout_line(
    trimmed: &str,
    plugins: &PluginMap,
    plugin_id: &str,
    plugin_name: &str,
    event_tx: &mpsc::Sender<(String, String, PluginEvent)>,
) {
    if trimmed.is_empty() {
        return;
    }
    let notification = match serde_json::from_str::<JsonRpcNotification>(trimmed) {
        Ok(n) if n.method == "event" => n,
        _ => return,
    };
    let event = match serde_json::from_value::<PluginEvent>(notification.params) {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut handles = plugins.write().await;
    if let Some(handle) = handles.get_mut(plugin_id) {
        handle.last_event_at = Some(Utc::now());
    }
    drop(handles);
    let _ = event_tx
        .send((plugin_id.to_string(), plugin_name.to_string(), event))
        .await;
}
