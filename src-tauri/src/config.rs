use crate::error::AppResult;
use crate::models::{PluginConfig, ScheduleConfig, SubscriptionConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
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
    #[serde(default)]
    pub schedules: Vec<ScheduleConfig>,
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
    #[serde(default)]
    pub subscriptions: Vec<SubscriptionConfig>,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_true")]
    pub enable_on_startup: bool,
}

fn default_timeout_secs() -> u64 {
    3600
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ts_rs::TS)]
#[ts(export)]
pub enum TerminalType {
    Auto,
    Pwsh,
    PowerShell,
    Cmd,
    Wsl,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, ts_rs::TS)]
#[ts(export)]
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
            schedules: Vec::new(),
            plugins: Vec::new(),
            subscriptions: Vec::new(),
            timeout_secs: 3600,
            enable_on_startup: true,
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
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Failed to parse config {}: {}", path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read config {}: {}", path.display(), e);
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> AppResult<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = AppConfig::default();
        assert_eq!(config.shortcut, "Ctrl+Shift+Space");
        assert_eq!(config.terminal, TerminalType::Auto);
        assert_eq!(config.wsl_shell, WslShell::Bash);
        assert!(config.last_directory.is_none());
        assert!(config.recent_directories.is_empty());
        assert!(config.schedules.is_empty());
        assert!(config.plugins.is_empty());
        assert!(config.subscriptions.is_empty());
    }

    #[test]
    fn serde_roundtrip() {
        let config = AppConfig {
            shortcut: "Ctrl+Alt+C".to_string(),
            terminal: TerminalType::Pwsh,
            wsl_shell: WslShell::Zsh,
            last_directory: Some("C:\\test".to_string()),
            recent_directories: vec!["C:\\test".to_string()],
            wsl_directory: Some("/home/user".to_string()),
            wsl_recent_directories: vec!["/home/user".to_string()],
            schedules: Vec::new(),
            plugins: Vec::new(),
            subscriptions: Vec::new(),
            timeout_secs: 3600,
            enable_on_startup: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.shortcut, "Ctrl+Alt+C");
        assert_eq!(restored.terminal, TerminalType::Pwsh);
        assert_eq!(restored.wsl_shell, WslShell::Zsh);
        assert_eq!(restored.last_directory, Some("C:\\test".to_string()));
    }

    #[test]
    fn missing_fields_use_defaults() {
        let json = r#"{"shortcut":"Ctrl+Space","terminal":"Cmd"}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.shortcut, "Ctrl+Space");
        assert_eq!(config.terminal, TerminalType::Cmd);
        assert_eq!(config.wsl_shell, WslShell::Bash);
        assert!(config.last_directory.is_none());
        assert!(config.recent_directories.is_empty());
    }
}
