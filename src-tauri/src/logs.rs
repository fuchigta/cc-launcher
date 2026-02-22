use crate::error::{AppError, AppResult};
use crate::models::ExecutionLog;
use std::fs;
use std::path::PathBuf;

fn logs_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cc-launcher")
        .join("logs")
}

pub fn write_log(log: &ExecutionLog) -> AppResult<()> {
    let dir = logs_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", log.id));
    let json = serde_json::to_string_pretty(log)?;
    fs::write(&path, json)?;
    Ok(())
}

pub fn list_logs(limit: usize, offset: usize) -> AppResult<Vec<ExecutionLog>> {
    let dir = logs_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

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

pub fn get_log(id: &str) -> AppResult<ExecutionLog> {
    let path = logs_dir().join(format!("{}.json", id));
    let content = fs::read_to_string(&path)
        .map_err(|e| AppError::NotFound(format!("Log not found: {}", e)))?;
    serde_json::from_str(&content).map_err(AppError::Json)
}

pub fn clear_logs() -> AppResult<()> {
    let dir = logs_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_log_not_found() {
        let result = get_log("nonexistent-id");
        assert!(result.is_err());
    }
}
