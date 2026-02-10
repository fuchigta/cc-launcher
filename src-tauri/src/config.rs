use crate::models::{PluginConfig, ScheduleConfig, SubscriptionConfig};
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
    #[serde(default)]
    pub schedules: Vec<ScheduleConfig>,
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
    #[serde(default)]
    pub subscriptions: Vec<SubscriptionConfig>,
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
            schedules: Vec::new(),
            plugins: Vec::new(),
            subscriptions: Vec::new(),
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
