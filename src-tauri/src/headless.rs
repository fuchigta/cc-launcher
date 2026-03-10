use crate::config::{AppConfig, TerminalType, WslShell};
use crate::error::AppResult;
use crate::logs;
use crate::models::{ExecutionLog, ExecutionSource, ExecutionStatus};
use chrono::Utc;
use tauri::Emitter;
use uuid::Uuid;

fn shell_command_str(prompt: &str, claude_args: &[String], terminal: &TerminalType) -> String {
    let args_str = claude_args.join(" ");
    match terminal {
        TerminalType::Wsl => {
            let escaped = prompt.replace("'", "'\\''");
            if args_str.is_empty() {
                format!("claude --print '{}'", escaped)
            } else {
                format!("claude --print '{}' {}", escaped, args_str)
            }
        }
        TerminalType::Cmd => {
            let escaped = crate::terminal::escape_cmd_meta(prompt);
            if args_str.is_empty() {
                format!("claude --print \"{}\"", escaped)
            } else {
                format!("claude --print \"{}\" {}", escaped, args_str)
            }
        }
        _ => {
            // PowerShell / Pwsh
            let escaped = prompt.replace("'", "''");
            if args_str.is_empty() {
                format!("claude --print '{}'", escaped)
            } else {
                format!("claude --print '{}' {}", escaped, args_str)
            }
        }
    }
}

fn build_shell_command(
    prompt: &str,
    claude_args: &[String],
    terminal: &TerminalType,
    wsl_shell: &WslShell,
    working_dir: Option<&str>,
) -> std::process::Command {
    let claude_cmd = shell_command_str(prompt, claude_args, terminal);
    match terminal {
        TerminalType::Wsl => {
            let shell_name = crate::terminal::wsl_shell_name(wsl_shell);
            let mut cmd = crate::windows_util::no_window_command("wsl");
            if let Some(dir) = working_dir {
                cmd.args(["--cd", dir]);
            }
            cmd.args(["--", shell_name, "-l", "-i", "-c", &claude_cmd]);
            cmd
        }
        TerminalType::Cmd => {
            let mut cmd = crate::windows_util::no_window_command("cmd");
            cmd.args(["/c", &claude_cmd]);
            cmd
        }
        _ => {
            // PowerShell / Pwsh（resolve済みなら Auto は来ない）
            let shell = if *terminal == TerminalType::PowerShell {
                "powershell"
            } else {
                "pwsh"
            };
            let mut cmd = crate::windows_util::no_window_command(shell);
            cmd.args(["-NoProfile", "-Command", &claude_cmd]);
            cmd
        }
    }
}

pub async fn execute(
    prompt: &str,
    working_dir: Option<&str>,
    claude_args: &[String],
    source: ExecutionSource,
    app_handle: &tauri::AppHandle,
) -> AppResult<ExecutionLog> {
    let id = Uuid::new_v4().to_string();
    let started_at = Utc::now();

    // Running ログを書き込んで execution-started イベントを発火
    let running_log = ExecutionLog {
        id: id.clone(),
        source: source.clone(),
        prompt: prompt.to_string(),
        working_dir: working_dir.map(|s| s.to_string()),
        claude_args: claude_args.to_vec(),
        status: ExecutionStatus::Running,
        stdout: String::new(),
        stderr: String::new(),
        exit_code: None,
        started_at,
        completed_at: None,
        duration_ms: None,
    };
    logs::write_log(&running_log).ok();
    let _ = app_handle.emit("execution-started", &running_log);

    let config = AppConfig::load();
    let resolved_terminal = crate::terminal::TerminalDetector::resolve(&config.terminal);

    let mut std_cmd = build_shell_command(
        prompt,
        claude_args,
        &resolved_terminal,
        &config.wsl_shell,
        working_dir,
    );

    std_cmd.stdout(std::process::Stdio::piped());
    std_cmd.stderr(std::process::Stdio::piped());

    // WSL 以外は current_dir で作業ディレクトリを設定（WSL は --cd で対処済み）
    if resolved_terminal != TerminalType::Wsl {
        let effective_dir = working_dir
            .map(std::path::PathBuf::from)
            .or_else(crate::windows_util::default_working_dir);
        if let Some(dir) = effective_dir {
            std_cmd.current_dir(dir);
        }
    }

    let mut cmd = tokio::process::Command::from(std_cmd);

    let timeout_secs = config.timeout_secs;
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let timeout_result = tokio::time::timeout(timeout_dur, cmd.output()).await;

    let completed_at = Utc::now();
    let duration_ms = (completed_at - started_at).num_milliseconds() as u64;

    let (status, stdout, stderr, exit_code) = match timeout_result {
        Err(_) => (
            ExecutionStatus::Failed,
            String::new(),
            format!("Claude execution timed out ({}s)", timeout_secs),
            None,
        ),
        Ok(Err(e)) => (
            ExecutionStatus::Failed,
            String::new(),
            format!("Failed to execute claude: {}", e),
            None,
        ),
        Ok(Ok(output)) => {
            let status = if output.status.success() {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failed
            };
            (
                status,
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
                output.status.code(),
            )
        }
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
