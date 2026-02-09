use crate::logs;
use crate::models::{ExecutionLog, ExecutionSource, ExecutionStatus};
use chrono::Utc;
use tauri::Emitter;
use uuid::Uuid;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn execute(
    prompt: &str,
    working_dir: Option<&str>,
    claude_args: &[String],
    source: ExecutionSource,
    app_handle: &tauri::AppHandle,
) -> Result<ExecutionLog, String> {
    let id = Uuid::new_v4().to_string();
    let started_at = Utc::now();

    let mut std_cmd = std::process::Command::new("claude");
    std_cmd.arg("--print");
    std_cmd.arg(prompt);
    for arg in claude_args {
        std_cmd.arg(arg);
    }

    use std::os::windows::process::CommandExt;
    std_cmd.creation_flags(CREATE_NO_WINDOW);

    std_cmd.stdout(std::process::Stdio::piped());
    std_cmd.stderr(std::process::Stdio::piped());

    if let Some(dir) = working_dir {
        std_cmd.current_dir(dir);
    }

    let mut cmd = tokio::process::Command::from(std_cmd);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute claude: {}", e))?;

    let completed_at = Utc::now();
    let duration_ms = (completed_at - started_at).num_milliseconds() as u64;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();

    let status = if output.status.success() {
        ExecutionStatus::Success
    } else {
        ExecutionStatus::Failed
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

    // Send notification
    send_notification(app_handle, &log);

    // Emit event to frontend
    let _ = app_handle.emit("execution-completed", &log);

    Ok(log)
}

fn send_notification(app_handle: &tauri::AppHandle, log: &ExecutionLog) {
    use tauri_plugin_notification::NotificationExt;

    let title = match &log.status {
        ExecutionStatus::Success => "Claude Code: Success",
        ExecutionStatus::Failed => "Claude Code: Failed",
        ExecutionStatus::Running => "Claude Code: Running",
    };

    let body = if log.stdout.len() > 200 {
        format!("{}...", &log.stdout[..200])
    } else if log.stdout.is_empty() {
        log.prompt.clone()
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
