use crate::config::TerminalType;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub terminal_type: TerminalType,
    pub display_name: String,
    pub available: bool,
}

pub struct TerminalDetector;

impl TerminalDetector {
    pub fn detect_available() -> Vec<TerminalInfo> {
        let mut terminals = Vec::new();

        // pwsh (PowerShell 7+)
        let pwsh_available = Command::new("pwsh")
            .arg("--version")
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

pub fn launch_claude(terminal: &TerminalType, prompt: &str) -> Result<(), String> {
    let escaped_prompt = prompt.replace("\"", "\\\"");

    let resolved_terminal = if *terminal == TerminalType::Auto {
        TerminalDetector::get_best()
    } else {
        terminal.clone()
    };

    match resolved_terminal {
        TerminalType::Pwsh | TerminalType::Auto => {
            Command::new("cmd")
                .args([
                    "/c",
                    "start",
                    "pwsh",
                    "-NoExit",
                    "-Command",
                    &format!("claude \"{}\"", escaped_prompt),
                ])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        TerminalType::PowerShell => {
            Command::new("cmd")
                .args([
                    "/c",
                    "start",
                    "powershell",
                    "-NoExit",
                    "-Command",
                    &format!("claude \"{}\"", escaped_prompt),
                ])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        TerminalType::Cmd => {
            Command::new("cmd")
                .args([
                    "/c",
                    "start",
                    "cmd",
                    "/k",
                    &format!("claude \"{}\"", escaped_prompt),
                ])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
