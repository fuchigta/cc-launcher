use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub shortcut: String,
    pub terminal: TerminalType,
    #[serde(default)]
    pub wsl_shell: WslShell,
    #[serde(default)]
    pub last_directory: Option<String>,
    #[serde(default)]
    pub recent_directories: Vec<String>,
    #[serde(default)]
    pub wsl_directory: Option<String>,
    #[serde(default)]
    pub wsl_recent_directories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminalType {
    Auto,
    Pwsh,
    PowerShell,
    Cmd,
    Wsl,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum WslShell {
    #[default]
    Bash,
    Zsh,
    Sh,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: "Ctrl+Shift+Space".to_string(),
            terminal: TerminalType::Auto,
            wsl_shell: WslShell::default(),
            last_directory: None,
            recent_directories: Vec::new(),
            wsl_directory: None,
            wsl_recent_directories: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cc-launcher")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }
}
