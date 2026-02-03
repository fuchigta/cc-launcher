use crate::config::{TerminalType, WslShell};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

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
                } else if lower.contains("powershell") {
                    return TerminalType::PowerShell;
                } else if lower.contains("cmd") {
                    return TerminalType::Cmd;
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

pub struct TerminalDetector;

impl TerminalDetector {
    pub fn detect_available() -> Vec<TerminalInfo> {
        let mut terminals = Vec::new();

        // pwsh (PowerShell 7+)
        let pwsh_available = Command::new("pwsh")
            .arg("--version")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        terminals.push(TerminalInfo {
            terminal_type: TerminalType::Pwsh,
            display_name: "PowerShell 7+ (pwsh)".to_string(),
            available: pwsh_available,
        });

        // powershell (Windows PowerShell)
        let powershell_available = Command::new("powershell")
            .arg("-Command")
            .arg("$PSVersionTable.PSVersion")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        terminals.push(TerminalInfo {
            terminal_type: TerminalType::PowerShell,
            display_name: "Windows PowerShell".to_string(),
            available: powershell_available,
        });

        // cmd is always available on Windows
        terminals.push(TerminalInfo {
            terminal_type: TerminalType::Cmd,
            display_name: "Command Prompt (cmd)".to_string(),
            available: true,
        });

        // WSL
        let wsl_available = Command::new("wsl")
            .arg("--status")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        terminals.push(TerminalInfo {
            terminal_type: TerminalType::Wsl,
            display_name: "WSL".to_string(),
            available: wsl_available,
        });

        terminals
    }

    pub fn get_best() -> TerminalType {
        let terminals = Self::detect_available();
        for t in terminals {
            if t.available && t.terminal_type != TerminalType::Cmd {
                return t.terminal_type;
            }
        }
        TerminalType::Cmd
    }

    pub fn resolve(config_terminal: &TerminalType) -> TerminalType {
        match config_terminal {
            TerminalType::Auto => Self::get_best(),
            other => other.clone(),
        }
    }
}

pub fn launch_claude(
    terminal: &TerminalType,
    prompt: &str,
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
    let mut args: Vec<String> = Vec::new();

    match resolved_terminal {
        TerminalType::Wsl => {
            // wsl_directoryが設定されている場合はそれを使用（パス変換なし）
            // 設定されていない場合は従来通りworking_dirをWSLパスに変換
            let wsl_path = wsl_directory
                .map(|s| s.to_string())
                .or_else(|| working_dir.and_then(windows_to_wsl_path));
            let escaped = prompt.replace("'", "'\\''");
            let claude_cmd = format!("claude '{}'", escaped);
            let shell_name = match wsl_shell {
                WslShell::Bash => "bash",
                WslShell::Zsh => "zsh",
                WslShell::Sh => "sh",
            };

            if let Some(wsl_dir) = wsl_path {
                args.extend([
                    "wsl".to_string(),
                    "--cd".to_string(),
                    wsl_dir,
                    "--".to_string(),
                    shell_name.to_string(),
                    "-l".to_string(),
                    "-i".to_string(),
                    "-c".to_string(),
                    claude_cmd,
                ]);
            } else {
                args.extend([
                    "wsl".to_string(),
                    "--".to_string(),
                    shell_name.to_string(),
                    "-l".to_string(),
                    "-i".to_string(),
                    "-c".to_string(),
                    claude_cmd,
                ]);
            }
        }
        TerminalType::Pwsh | TerminalType::Auto => {
            let escaped = prompt.replace("'", "''");
            let claude_cmd = format!("claude '{}'", escaped);
            if let Some(dir) = working_dir {
                args.extend(["-d".to_string(), dir.to_string()]);
            }
            args.extend([
                "--".to_string(),
                "pwsh".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                claude_cmd,
            ]);
        }
        TerminalType::PowerShell => {
            let escaped = prompt.replace("'", "''");
            let claude_cmd = format!("claude '{}'", escaped);
            if let Some(dir) = working_dir {
                args.extend(["-d".to_string(), dir.to_string()]);
            }
            args.extend([
                "--".to_string(),
                "powershell".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                claude_cmd,
            ]);
        }
        TerminalType::Cmd => {
            let escaped = prompt.replace("\"", "\\\"").replace("%", "%%");
            let claude_cmd = format!("claude \"{}\"", escaped);
            if let Some(dir) = working_dir {
                args.extend(["-d".to_string(), dir.to_string()]);
            }
            args.extend([
                "--".to_string(),
                "cmd".to_string(),
                "/k".to_string(),
                claude_cmd,
            ]);
        }
    }

    cmd.args(&args);
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}
