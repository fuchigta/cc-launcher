use crate::error::{AppError, AppResult};
use crate::logs;
use crate::models::{ExecutionLog, ExecutionSource, ExecutionStatus};
use chrono::Utc;
use tauri::Emitter;
use uuid::Uuid;

pub async fn execute(
    prompt: &str,
    working_dir: Option<&str>,
    claude_args: &[String],
    source: ExecutionSource,
    app_handle: &tauri::AppHandle,
) -> AppResult<ExecutionLog> {
    let id = Uuid::new_v4().to_string();
    let started_at = Utc::now();

    let mut std_cmd = crate::windows_util::no_window_command("claude");
    std_cmd.arg("--print");
    std_cmd.arg(prompt);
    for arg in claude_args {
        std_cmd.arg(arg);
    }

    std_cmd.stdout(std::process::Stdio::piped());
    std_cmd.stderr(std::process::Stdio::piped());

    if let Some(dir) = working_dir {
        std_cmd.current_dir(dir);
    }

    let mut cmd = tokio::process::Command::from(std_cmd);

    let timeout_secs = std::time::Duration::from_secs(300);
    let output = tokio::time::timeout(timeout_secs, cmd.output())
        .await
        .map_err(|_| AppError::Execution("Claude execution timed out (300s)".to_string()))?
        .map_err(|e| AppError::Execution(format!("Failed to execute claude: {}", e)))?;

    let completed_at = Utc::now();
    let duration_ms = (completed_at - started_at).num_milliseconds() as u64;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();

    let status = match output.status.success() {
        true => ExecutionStatus::Success,
        false => ExecutionStatus::Failed,
    };

    let log = ExecutionLog {
        id,
        source,
        prompt: prompt.to_string(),
        working_dir: working_dir.map(|s| s.to_string()),
        claude_args: claude_args.to_vec(),
        status,
        stdout,
        stderr,
        exit_code,
        started_at,
        completed_at: Some(completed_at),
        duration_ms: Some(duration_ms),
    };

    logs::write_log(&log).ok();
    send_notification(app_handle, &log);
    let _ = app_handle.emit("execution-completed", &log);

    Ok(log)
}

fn send_notification(app_handle: &tauri::AppHandle, log: &ExecutionLog) {
    use tauri_plugin_notification::NotificationExt;

    let title = match log.status {
        ExecutionStatus::Success => "Claude Code: Success",
        ExecutionStatus::Failed => "Claude Code: Failed",
        ExecutionStatus::Running => "Claude Code: Running",
    };

    let body = if log.stdout.is_empty() {
        log.prompt.clone()
    } else if log.stdout.len() > 200 {
        let end = log.stdout.floor_char_boundary(200);
        format!("{}...", &log.stdout[..end])
    } else {
        log.stdout.clone()
    };

    let _ = app_handle
        .notification()
        .builder()
        .title(title)
        .body(&body)
        .show();
}
