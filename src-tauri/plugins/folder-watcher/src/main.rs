use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use chrono::Utc;
use clap::Parser;
use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// --- CLI ---

#[derive(Parser)]
#[command(name = "folder-watcher")]
struct Cli {
    /// Directory to watch
    #[arg(long)]
    dir: String,

    /// Watch subdirectories recursively
    #[arg(long, default_value_t = false)]
    recursive: bool,

    /// Comma-separated glob patterns to include (e.g. "*.txt,*.csv")
    #[arg(long)]
    filter: Option<String>,

    /// Comma-separated patterns to ignore (e.g. ".git,node_modules")
    #[arg(long, default_value = ".git,node_modules")]
    ignore: String,

    /// Debounce interval in milliseconds
    #[arg(long, default_value_t = 300)]
    debounce: u64,
}

// --- JSON-RPC types ---

#[derive(Deserialize)]
struct JsonRpcMessage {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[allow(dead_code)]
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    result: Value,
}

#[derive(Serialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: Value,
}

// --- Helpers ---

fn send_json(value: &impl Serialize) {
    let line = serde_json::to_string(value).expect("serialize");
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(line.as_bytes());
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}

fn send_response(id: Value, result: Value) {
    send_json(&JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result,
    });
}

fn send_event(event_type: &str, data: Value) {
    send_json(&JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "event".to_string(),
        params: serde_json::json!({
            "eventType": event_type,
            "data": data,
        }),
    });
}

fn build_ignore_set(ignore: &str) -> Vec<String> {
    ignore
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn build_filter_globset(filter: &Option<String>) -> Option<GlobSet> {
    let filter_str = filter.as_deref()?;
    let mut builder = GlobSetBuilder::new();
    for pattern in filter_str.split(',') {
        let pattern = pattern.trim();
        if !pattern.is_empty() {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
    }
    builder.build().ok()
}

fn should_ignore(path: &Path, ignore_patterns: &[String]) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        ignore_patterns.iter().any(|p| name.as_ref() == p.as_str())
    })
}

fn matches_filter(path: &Path, watch_dir: &Path, filter_set: &Option<GlobSet>) -> bool {
    match filter_set {
        Some(set) => {
            let relative = path.strip_prefix(watch_dir).unwrap_or(path);
            set.is_match(relative)
        }
        None => true,
    }
}

// --- Debounce logic ---

struct Debouncer {
    pending: HashMap<PathBuf, (String, Value, std::time::Instant)>,
    interval: Duration,
}

impl Debouncer {
    fn new(interval_ms: u64) -> Self {
        Self {
            pending: HashMap::new(),
            interval: Duration::from_millis(interval_ms),
        }
    }

    fn add(&mut self, path: PathBuf, event_type: String, data: Value) {
        self.pending
            .insert(path, (event_type, data, std::time::Instant::now()));
    }

    fn flush(&mut self) {
        let now = std::time::Instant::now();
        let ready: Vec<PathBuf> = self
            .pending
            .iter()
            .filter(|(_, (_, _, t))| now.duration_since(*t) >= self.interval)
            .map(|(k, _)| k.clone())
            .collect();

        for path in ready {
            if let Some((event_type, data, _)) = self.pending.remove(&path) {
                send_event(&event_type, data);
            }
        }
    }
}

// --- Rename tracking ---

struct RenameTracker {
    pending_remove: Option<(PathBuf, std::time::Instant)>,
}

impl RenameTracker {
    fn new() -> Self {
        Self {
            pending_remove: None,
        }
    }

    fn on_remove(&mut self, path: PathBuf) {
        self.pending_remove = Some((path, std::time::Instant::now()));
    }

    fn on_create(&mut self, _path: &Path) -> Option<PathBuf> {
        if let Some((old_path, instant)) = self.pending_remove.take() {
            if instant.elapsed() < Duration::from_millis(100) {
                return Some(old_path);
            }
            // Too old, treat as separate remove+create
            send_event(
                "file_deleted",
                serde_json::json!({
                    "file_path": old_path.to_string_lossy(),
                    "timestamp": Utc::now().to_rfc3339(),
                }),
            );
        }
        None
    }

    fn flush(&mut self) {
        if let Some((old_path, instant)) = self.pending_remove.take() {
            if instant.elapsed() >= Duration::from_millis(100) {
                send_event(
                    "file_deleted",
                    serde_json::json!({
                        "file_path": old_path.to_string_lossy(),
                        "timestamp": Utc::now().to_rfc3339(),
                    }),
                );
            } else {
                self.pending_remove = Some((old_path, instant));
            }
        }
    }
}

// --- Main ---

fn main() {
    let cli = Cli::parse();

    let watch_dir = PathBuf::from(&cli.dir);
    if !watch_dir.exists() {
        eprintln!("Directory not found: {}", cli.dir);
        std::process::exit(1);
    }

    let ignore_patterns = build_ignore_set(&cli.ignore);
    let filter_set = build_filter_globset(&cli.filter);
    let recursive = cli.recursive;
    let debounce_ms = cli.debounce;

    // Read JSON-RPC messages from stdin
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg: JsonRpcMessage = match serde_json::from_str(trimmed) {
            Ok(m) => m,
            Err(_) => continue,
        };

        match msg.method.as_str() {
            "initialize" => {
                if let Some(id) = msg.id {
                    send_response(
                        id,
                        serde_json::json!({
                            "name": "folder-watcher",
                            "version": "0.1.0",
                            "description": "Watches a directory for file changes (built-in)",
                            "watchDir": watch_dir.to_string_lossy(),
                            "recursive": recursive,
                        }),
                    );
                }

                // Start watching in a separate thread
                let watch_dir2 = watch_dir.clone();
                let ignore2 = ignore_patterns.clone();
                let filter2 = filter_set.clone();

                std::thread::spawn(move || {
                    run_watcher(&watch_dir2, recursive, debounce_ms, &ignore2, &filter2);
                });
            }
            "shutdown" => {
                if let Some(id) = msg.id {
                    send_response(id, serde_json::json!({"status": "ok"}));
                }
                std::process::exit(0);
            }
            _ => {}
        }
    }

    // stdin closed
    std::process::exit(0);
}

fn run_watcher(
    watch_dir: &Path,
    recursive: bool,
    debounce_ms: u64,
    ignore_patterns: &[String],
    filter_set: &Option<GlobSet>,
) {
    let (tx, rx) = std_mpsc::channel();

    let mut watcher = match notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to create watcher: {}", e);
            return;
        }
    };

    let mode = if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    if let Err(e) = watcher.watch(watch_dir, mode) {
        eprintln!("Failed to watch directory: {}", e);
        return;
    }

    let mut debouncer = Debouncer::new(debounce_ms);
    let mut rename_tracker = RenameTracker::new();

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                for path in &event.paths {
                    if should_ignore(path, ignore_patterns) {
                        continue;
                    }
                    if !matches_filter(path, watch_dir, filter_set) {
                        continue;
                    }

                    let timestamp = Utc::now().to_rfc3339();

                    match event.kind {
                        EventKind::Create(_) => {
                            if let Some(old_path) = rename_tracker.on_create(path) {
                                // This is a rename (remove followed by create)
                                send_event(
                                    "file_renamed",
                                    serde_json::json!({
                                        "old_path": old_path.to_string_lossy(),
                                        "new_path": path.to_string_lossy(),
                                        "timestamp": timestamp,
                                    }),
                                );
                            } else {
                                debouncer.add(
                                    path.clone(),
                                    "file_created".to_string(),
                                    serde_json::json!({
                                        "file_path": path.to_string_lossy(),
                                        "timestamp": timestamp,
                                    }),
                                );
                            }
                        }
                        EventKind::Modify(_) => {
                            debouncer.add(
                                path.clone(),
                                "file_changed".to_string(),
                                serde_json::json!({
                                    "file_path": path.to_string_lossy(),
                                    "timestamp": timestamp,
                                }),
                            );
                        }
                        EventKind::Remove(_) => {
                            rename_tracker.on_remove(path.clone());
                        }
                        _ => {}
                    }
                }
            }
            Err(std_mpsc::RecvTimeoutError::Timeout) => {}
            Err(std_mpsc::RecvTimeoutError::Disconnected) => break,
        }

        debouncer.flush();
        rename_tracker.flush();
    }
}
