use crate::config::{TerminalType, WslShell};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub terminal_type: TerminalType,
    pub display_name: String,
    pub available: bool,
}

/// Windows Terminalの設定ファイルパスを取得
fn get_wt_settings_path() -> Option<PathBuf> {
    let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
    let path = PathBuf::from(&local_app_data)
        .join("Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState/settings.json");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// 既定プロファイルのシェル種類を検出
fn detect_wt_default_shell() -> TerminalType {
    let Some(settings_path) = get_wt_settings_path() else {
        return TerminalType::PowerShell;
    };
    let Ok(content) = fs::read_to_string(&settings_path) else {
        return TerminalType::PowerShell;
    };
    let Ok(settings) = serde_json::from_str::<Value>(&content) else {
        return TerminalType::PowerShell;
    };

    let Some(default_profile) = settings.get("defaultProfile").and_then(|v| v.as_str()) else {
        return TerminalType::PowerShell;
    };
    let Some(profiles) = settings
        .get("profiles")
        .and_then(|p| p.get("list"))
        .and_then(|l| l.as_array())
    else {
        return TerminalType::PowerShell;
    };

    for profile in profiles {
        let guid = profile.get("guid").and_then(|v| v.as_str());
        let name = profile.get("name").and_then(|v| v.as_str());

        if guid == Some(default_profile) || name == Some(default_profile) {
            // sourceでWSLを判定
            if let Some(source) = profile.get("source").and_then(|v| v.as_str()) {
                if source.contains("WSL") || source.contains("wsl") {
                    return TerminalType::Wsl;
                }
                if source.contains("PowerShell") {
                    return TerminalType::Pwsh;
                }
            }
            // commandlineで判定
            if let Some(commandline) = profile.get("commandline").and_then(|v| v.as_str()) {
                let lower = commandline.to_lowercase();
                if lower.contains("wsl") {
                    return TerminalType::Wsl;
                } else if lower.contains("pwsh") {
                    return TerminalType::Pwsh;
                } else if lower.contains("powershell") || lower.contains("cmd") {
                    return TerminalType::PowerShell;
                }
            }
        }
    }

    TerminalType::PowerShell
}

/// WindowsパスをWSLパスに変換
fn windows_to_wsl_path(windows_path: &str) -> Option<String> {
    let path = windows_path.replace("\\", "/");
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        let drive = path.chars().next()?.to_lowercase().next()?;
        let rest = &path[2..];
        Some(format!("/mnt/{}{}", drive, rest))
    } else {
        None
    }
}

fn command_available(cmd: &str, args: &[&str]) -> bool {
    crate::windows_util::no_window_command(cmd)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub struct TerminalDetector;

impl TerminalDetector {
    pub fn detect_available() -> Vec<TerminalInfo> {
        vec![
            TerminalInfo {
                terminal_type: TerminalType::Pwsh,
                display_name: "PowerShell 7+ (pwsh)".to_string(),
                available: command_available("pwsh", &["--version"]),
            },
            TerminalInfo {
                terminal_type: TerminalType::PowerShell,
                display_name: "Windows PowerShell".to_string(),
                available: command_available(
                    "powershell",
                    &["-Command", "$PSVersionTable.PSVersion"],
                ),
            },
            TerminalInfo {
                terminal_type: TerminalType::Wsl,
                display_name: "WSL".to_string(),
                available: command_available("wsl", &["--status"]),
            },
        ]
    }

    pub fn get_best() -> TerminalType {
        let terminals = Self::detect_available();
        for t in terminals {
            if t.available {
                return t.terminal_type;
            }
        }
        TerminalType::PowerShell
    }

    pub fn resolve(config_terminal: &TerminalType) -> TerminalType {
        match config_terminal {
            TerminalType::Auto => Self::get_best(),
            other => other.clone(),
        }
    }
}

fn cursor_monitor_pos_args() -> Vec<String> {
    if let Some((mon_x, mon_y, _mon_w, _mon_h)) =
        crate::windows_util::get_cursor_monitor_work_area()
    {
        let x = mon_x + 50;
        let y = mon_y + 50;
        vec!["--pos".to_string(), format!("{},{}", x, y)]
    } else {
        vec![]
    }
}

pub(crate) fn normalize_line_endings(prompt: &str) -> String {
    prompt.replace("\r\n", "\n").replace('\r', "\n")
}

/// PowerShell の -EncodedCommand 用に UTF-16LE + Base64 エンコードする
pub(crate) fn encode_powershell_command(cmd: &str) -> String {
    use base64::Engine;
    let utf16le: Vec<u8> = cmd.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    base64::engine::general_purpose::STANDARD.encode(&utf16le)
}

/// bash $'...' (ANSI-C quoting) 用エスケープ
/// \\ → \\\\, ' → \', 改行 → \n
pub(crate) fn escape_bash_dollar_quote(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '\'' => result.push_str("\\'"),
            '\n' => result.push_str("\\n"),
            _ => result.push(c),
        }
    }
    result
}

pub(crate) fn wsl_shell_name(shell: &crate::config::WslShell) -> &'static str {
    match shell {
        crate::config::WslShell::Bash => "bash",
        crate::config::WslShell::Zsh => "zsh",
        crate::config::WslShell::Sh => "sh",
    }
}

pub fn launch_claude(
    terminal: &TerminalType,
    prompt: &str,
    working_dir: Option<&str>,
    wsl_shell: &WslShell,
    wsl_directory: Option<&str>,
) -> Result<(), String> {
    let prompt = normalize_line_endings(prompt);

    let resolved_terminal = if *terminal == TerminalType::Auto {
        detect_wt_default_shell()
    } else {
        terminal.clone()
    };

    let mut cmd = Command::new("wt");
    let mut args: Vec<String> = cursor_monitor_pos_args();

    match resolved_terminal {
        TerminalType::Wsl => {
            // wsl_directoryが設定されている場合はそれを使用（パス変換なし）
            // 設定されていない場合は従来通りworking_dirをWSLパスに変換
            let wsl_path = wsl_directory
                .map(|s| s.to_string())
                .or_else(|| working_dir.and_then(windows_to_wsl_path));
            let escaped = escape_bash_dollar_quote(&prompt);
            let claude_cmd = format!("claude $'{}'", escaped);
            let shell_name = wsl_shell_name(wsl_shell);

            args.push("wsl".to_string());
            if let Some(wsl_dir) = wsl_path {
                args.extend(["--cd".to_string(), wsl_dir]);
            }
            args.extend([
                "--".to_string(),
                shell_name.to_string(),
                "-l".to_string(),
                "-i".to_string(),
                "-c".to_string(),
                claude_cmd,
            ]);
        }
        TerminalType::Pwsh | TerminalType::PowerShell | TerminalType::Auto => {
            let shell = if resolved_terminal == TerminalType::PowerShell {
                "powershell"
            } else {
                "pwsh"
            };
            let escaped = prompt.replace("'", "''");
            let claude_cmd = format!("claude '{}'", escaped);
            let encoded = encode_powershell_command(&claude_cmd);
            let effective_dir = working_dir.map(|s| s.to_string()).or_else(|| {
                crate::windows_util::default_working_dir()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
            });
            if let Some(dir) = effective_dir {
                args.extend(["-d".to_string(), dir]);
            }
            args.extend([
                "--".to_string(),
                shell.to_string(),
                "-NoExit".to_string(),
                "-EncodedCommand".to_string(),
                encoded,
            ]);
        }
    }

    cmd.args(&args);
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn resume_claude(
    terminal: &TerminalType,
    session_id: &str,
    working_dir: Option<&str>,
    wsl_shell: &WslShell,
    wsl_directory: Option<&str>,
) -> Result<(), String> {
    let resolved_terminal = if *terminal == TerminalType::Auto {
        detect_wt_default_shell()
    } else {
        terminal.clone()
    };

    let mut cmd = Command::new("wt");
    let mut args: Vec<String> = cursor_monitor_pos_args();

    match resolved_terminal {
        TerminalType::Wsl => {
            let wsl_path = wsl_directory
                .map(|s| s.to_string())
                .or_else(|| working_dir.and_then(windows_to_wsl_path));
            let claude_cmd = format!("claude --resume '{}'", session_id);
            let shell_name = match wsl_shell {
                WslShell::Bash => "bash",
                WslShell::Zsh => "zsh",
                WslShell::Sh => "sh",
            };

            args.push("wsl".to_string());
            if let Some(wsl_dir) = wsl_path {
                args.extend(["--cd".to_string(), wsl_dir]);
            }
            args.extend([
                "--".to_string(),
                shell_name.to_string(),
                "-l".to_string(),
                "-i".to_string(),
                "-c".to_string(),
                claude_cmd,
            ]);
        }
        TerminalType::Pwsh | TerminalType::PowerShell | TerminalType::Auto => {
            let shell = if resolved_terminal == TerminalType::PowerShell {
                "powershell"
            } else {
                "pwsh"
            };
            let claude_cmd = format!("claude --resume '{}'", session_id);
            let encoded = encode_powershell_command(&claude_cmd);
            let effective_dir = working_dir.map(|s| s.to_string()).or_else(|| {
                crate::windows_util::default_working_dir()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
            });
            if let Some(dir) = effective_dir {
                args.extend(["-d".to_string(), dir]);
            }
            args.extend([
                "--".to_string(),
                shell.to_string(),
                "-NoExit".to_string(),
                "-EncodedCommand".to_string(),
                encoded,
            ]);
        }
    }

    cmd.args(&args);
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_line_endings_preserves_plain_text() {
        assert_eq!(normalize_line_endings("hello world"), "hello world");
    }

    #[test]
    fn normalize_line_endings_preserves_lf() {
        assert_eq!(normalize_line_endings("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn normalize_line_endings_converts_crlf_to_lf() {
        assert_eq!(normalize_line_endings("line1\r\nline2"), "line1\nline2");
    }

    #[test]
    fn normalize_line_endings_converts_cr_to_lf() {
        assert_eq!(normalize_line_endings("line1\rline2"), "line1\nline2");
    }

    #[test]
    fn normalize_line_endings_converts_mixed() {
        assert_eq!(normalize_line_endings("a\r\nb\nc\rd"), "a\nb\nc\nd");
    }

    #[test]
    fn encode_powershell_command_roundtrip() {
        use base64::Engine;
        let original = "claude 'line1\nline2'";
        let encoded = encode_powershell_command(original);
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        let utf16: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let decoded = String::from_utf16(&utf16).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn escape_bash_dollar_quote_no_special() {
        assert_eq!(escape_bash_dollar_quote("hello world"), "hello world");
    }

    #[test]
    fn escape_bash_dollar_quote_newline() {
        assert_eq!(escape_bash_dollar_quote("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn escape_bash_dollar_quote_single_quote() {
        assert_eq!(escape_bash_dollar_quote("it's"), "it\\'s");
    }

    #[test]
    fn escape_bash_dollar_quote_backslash() {
        assert_eq!(escape_bash_dollar_quote("a\\b"), "a\\\\b");
    }

    #[test]
    fn escape_bash_dollar_quote_combined() {
        assert_eq!(escape_bash_dollar_quote("it's\na\\b"), "it\\'s\\na\\\\b");
    }
}
