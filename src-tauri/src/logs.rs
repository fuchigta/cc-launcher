use crate::models::ExecutionLog;
use std::fs;
use std::path::PathBuf;

fn logs_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cc-launcher")
        .join("logs")
}

pub fn write_log(log: &ExecutionLog) -> Result<(), String> {
    let dir = logs_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", log.id));
    let json = serde_json::to_string_pretty(log).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn list_logs(limit: usize, offset: usize) -> Result<Vec<ExecutionLog>, String> {
    let dir = logs_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    // Sort by modification time descending (newest first)
    entries.sort_by(|a, b| {
        let time_a = a.metadata().and_then(|m| m.modified()).ok();
        let time_b = b.metadata().and_then(|m| m.modified()).ok();
        time_b.cmp(&time_a)
    });

    let logs: Vec<ExecutionLog> = entries
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter_map(|entry| {
            let content = fs::read_to_string(entry.path()).ok()?;
            serde_json::from_str(&content).ok()
        })
        .collect();

    Ok(logs)
}

pub fn get_log(id: &str) -> Result<ExecutionLog, String> {
    let path = logs_dir().join(format!("{}.json", id));
    let content = fs::read_to_string(&path).map_err(|e| format!("Log not found: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse log: {}", e))
}

pub fn clear_logs() -> Result<(), String> {
    let dir = logs_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}
