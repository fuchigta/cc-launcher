use crate::config::{AppConfig, TerminalType, WslShell};
use crate::error::AppResult;
use crate::logs;
use crate::models::{ExecutionLog, ExecutionSource, ExecutionStatus};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

// --- Running execution registry ---

pub struct RunningExecution {
    pub pid: u32,
    pub cancel: Arc<Notify>,
}

pub struct RunningExecutionRegistry(pub Arc<Mutex<HashMap<String, RunningExecution>>>);

impl RunningExecutionRegistry {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

enum ShellCommand {
    /// 通常のコマンド文字列（-Command / -c に渡す）
    Plain(String),
    /// PowerShell -EncodedCommand 用の Base64 エンコード済み文字列
    Encoded(String),
}

fn shell_command_str(
    prompt: &str,
    claude_args: &[String],
    terminal: &TerminalType,
) -> ShellCommand {
    let prompt = crate::terminal::normalize_line_endings(prompt);
    let args_str = claude_args.join(" ");
    match terminal {
        TerminalType::Wsl => {
            let escaped = crate::terminal::escape_bash_dollar_quote(&prompt);
            let cmd = if args_str.is_empty() {
                format!("claude --print $'{}'", escaped)
            } else {
                format!("claude --print $'{}' {}", escaped, args_str)
            };
            ShellCommand::Plain(cmd)
        }
        _ => {
            // PowerShell / Pwsh
            let escaped = prompt.replace("'", "''");
            let ps_cmd = if args_str.is_empty() {
                format!("claude --print '{}'", escaped)
            } else {
                format!("claude --print '{}' {}", escaped, args_str)
            };
            ShellCommand::Encoded(crate::terminal::encode_powershell_command(&ps_cmd))
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
    let shell_cmd = shell_command_str(prompt, claude_args, terminal);
    match terminal {
        TerminalType::Wsl => {
            let shell_name = crate::terminal::wsl_shell_name(wsl_shell);
            let mut cmd = crate::windows_util::no_window_command("wsl");
            if let Some(dir) = working_dir {
                cmd.args(["--cd", dir]);
            }
            let ShellCommand::Plain(claude_cmd) = shell_cmd else {
                unreachable!("WSL always returns Plain")
            };
            cmd.args(["--", shell_name, "-l", "-i", "-c", &claude_cmd]);
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
            let ShellCommand::Encoded(encoded) = shell_cmd else {
                unreachable!("PowerShell always returns Encoded")
            };
            cmd.args(["-EncodedCommand", &encoded]);
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
    let session_id = Uuid::new_v4().to_string();
    let started_at = Utc::now();

    // --session-id をユーザー指定argsの前に追加
    let mut effective_args = vec!["--session-id".to_string(), session_id.clone()];
    effective_args.extend_from_slice(claude_args);

    // Running ログを書き込んで execution-started イベントを発火
    let running_log = ExecutionLog {
        id: id.clone(),
        session_id: Some(session_id.clone()),
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
        &effective_args,
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

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to execute claude: {}", e);
            let completed_at = Utc::now();
            let duration_ms = (completed_at - started_at).num_milliseconds() as u64;
            let log = ExecutionLog {
                id,
                session_id: Some(session_id),
                source,
                prompt: prompt.to_string(),
                working_dir: working_dir.map(|s| s.to_string()),
                claude_args: claude_args.to_vec(),
                status: ExecutionStatus::Failed,
                stdout: String::new(),
                stderr: err_msg,
                exit_code: None,
                started_at,
                completed_at: Some(completed_at),
                duration_ms: Some(duration_ms),
            };
            logs::write_log(&log).ok();
            send_notification(app_handle, &log);
            let _ = app_handle.emit("execution-completed", &log);
            return Ok(log);
        }
    };

    let pid = child.id().unwrap_or(0);

    let cancel_notify = Arc::new(Notify::new());
    if let Some(registry) = app_handle.try_state::<RunningExecutionRegistry>() {
        let mut guard = registry.0.lock().await;
        guard.insert(
            id.clone(),
            RunningExecution {
                pid,
                cancel: cancel_notify.clone(),
            },
        );
    }

    let timeout_secs = config.timeout_secs;
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);

    let (status, stdout, stderr, exit_code) = tokio::select! {
        result = child.wait_with_output() => {
            match result {
                Ok(output) => {
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
                Err(e) => (
                    ExecutionStatus::Failed,
                    String::new(),
                    format!("Failed to execute claude: {}", e),
                    None,
                ),
            }
        },
        _ = tokio::time::sleep(timeout_dur) => {
            crate::windows_util::kill_process_tree(pid);
            (
                ExecutionStatus::Failed,
                String::new(),
                format!("Claude execution timed out ({}s)", timeout_secs),
                None,
            )
        },
        _ = cancel_notify.notified() => {
            crate::windows_util::kill_process_tree(pid);
            (ExecutionStatus::Cancelled, String::new(), String::new(), None)
        },
    };

    // レジストリから除去
    if let Some(registry) = app_handle.try_state::<RunningExecutionRegistry>() {
        let mut guard = registry.0.lock().await;
        guard.remove(&id);
    }

    let completed_at = Utc::now();
    let duration_ms = (completed_at - started_at).num_milliseconds() as u64;

    let log = ExecutionLog {
        id,
        session_id: Some(session_id),
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

fn notification_body(prompt: &str, stdout: &str) -> String {
    if stdout.is_empty() {
        prompt.to_string()
    } else if stdout.len() > 200 {
        let end = stdout.floor_char_boundary(200);
        format!("{}...", &stdout[..end])
    } else {
        stdout.to_string()
    }
}

fn send_notification(app_handle: &tauri::AppHandle, log: &ExecutionLog) {
    use tauri_plugin_notification::NotificationExt;

    let title = match log.status {
        ExecutionStatus::Success => "Claude Code: Success",
        ExecutionStatus::Failed => "Claude Code: Failed",
        ExecutionStatus::Running => "Claude Code: Running",
        ExecutionStatus::Cancelled => "Claude Code: Cancelled",
    };

    let body = notification_body(&log.prompt, &log.stdout);

    let _ = app_handle
        .notification()
        .builder()
        .title(title)
        .body(&body)
        .show();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TerminalType;

    fn decode_powershell_encoded(encoded: &str) -> String {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        let utf16: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&utf16).unwrap()
    }

    #[test]
    fn shell_command_str_wsl_no_args() {
        let ShellCommand::Plain(cmd) = shell_command_str("hello world", &[], &TerminalType::Wsl)
        else {
            panic!("expected Plain");
        };
        assert_eq!(cmd, "claude --print $'hello world'");
    }

    #[test]
    fn shell_command_str_wsl_with_args() {
        let ShellCommand::Plain(cmd) = shell_command_str(
            "hello",
            &["--dangerously-skip-permissions".to_string()],
            &TerminalType::Wsl,
        ) else {
            panic!("expected Plain");
        };
        assert_eq!(
            cmd,
            "claude --print $'hello' --dangerously-skip-permissions"
        );
    }

    #[test]
    fn shell_command_str_wsl_escapes_single_quote() {
        let ShellCommand::Plain(cmd) = shell_command_str("it's a test", &[], &TerminalType::Wsl)
        else {
            panic!("expected Plain");
        };
        assert_eq!(cmd, "claude --print $'it\\'s a test'");
    }

    #[test]
    fn shell_command_str_wsl_multiline_prompt() {
        let ShellCommand::Plain(cmd) = shell_command_str("line1\nline2", &[], &TerminalType::Wsl)
        else {
            panic!("expected Plain");
        };
        assert_eq!(cmd, "claude --print $'line1\\nline2'");
    }

    #[test]
    fn shell_command_str_wsl_crlf_prompt() {
        let ShellCommand::Plain(cmd) = shell_command_str("line1\r\nline2", &[], &TerminalType::Wsl)
        else {
            panic!("expected Plain");
        };
        assert_eq!(cmd, "claude --print $'line1\\nline2'");
    }

    #[test]
    fn shell_command_str_powershell_no_args() {
        let ShellCommand::Encoded(enc) = shell_command_str("hello", &[], &TerminalType::PowerShell)
        else {
            panic!("expected Encoded");
        };
        assert_eq!(decode_powershell_encoded(&enc), "claude --print 'hello'");
    }

    #[test]
    fn shell_command_str_powershell_escapes_single_quote() {
        let ShellCommand::Encoded(enc) =
            shell_command_str("it's a test", &[], &TerminalType::PowerShell)
        else {
            panic!("expected Encoded");
        };
        assert_eq!(
            decode_powershell_encoded(&enc),
            "claude --print 'it''s a test'"
        );
    }

    #[test]
    fn shell_command_str_pwsh_no_args() {
        let ShellCommand::Encoded(enc) = shell_command_str("hello", &[], &TerminalType::Pwsh)
        else {
            panic!("expected Encoded");
        };
        assert_eq!(decode_powershell_encoded(&enc), "claude --print 'hello'");
    }

    #[test]
    fn shell_command_str_powershell_multiline_prompt() {
        let ShellCommand::Encoded(enc) =
            shell_command_str("line1\nline2", &[], &TerminalType::PowerShell)
        else {
            panic!("expected Encoded");
        };
        assert_eq!(
            decode_powershell_encoded(&enc),
            "claude --print 'line1\nline2'"
        );
    }

    #[test]
    fn notification_body_empty_stdout_returns_prompt() {
        assert_eq!(notification_body("my prompt", ""), "my prompt");
    }

    #[test]
    fn notification_body_short_stdout_returns_stdout() {
        assert_eq!(notification_body("prompt", "short output"), "short output");
    }

    #[test]
    fn notification_body_exactly_200_chars_not_truncated() {
        let exact = "a".repeat(200);
        let body = notification_body("prompt", &exact);
        assert_eq!(body, exact);
    }

    #[test]
    fn notification_body_long_stdout_truncates_with_ellipsis() {
        let long = "a".repeat(300);
        let body = notification_body("prompt", &long);
        assert!(body.ends_with("..."));
        assert_eq!(body, format!("{}...", "a".repeat(200)));
    }

    #[test]
    fn notification_body_long_multibyte_truncates_at_char_boundary() {
        // 各文字が3バイトのUTF-8文字（例: '€' = 3バイト）
        // 200バイト境界が文字の途中に来ないことを確認
        let long = "あ".repeat(100); // 100文字 × 3バイト = 300バイト
        let body = notification_body("prompt", &long);
        assert!(body.ends_with("..."));
        // パニックせず有効なUTF-8文字列であること
        assert!(std::str::from_utf8(body.as_bytes()).is_ok());
    }
}
