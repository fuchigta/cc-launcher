mod config;
mod terminal;

use config::AppConfig;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use terminal::TerminalInfo;

#[tauri::command]
fn get_config() -> AppConfig {
    AppConfig::load()
}

#[tauri::command]
fn save_config(new_config: AppConfig) -> Result<(), String> {
    new_config.save()
}

#[tauri::command]
fn get_available_terminals() -> Vec<TerminalInfo> {
    terminal::TerminalDetector::detect_available()
}

#[tauri::command]
async fn open_claude_interactive(
    prompt: String,
    working_dir: Option<String>,
) -> Result<(), String> {
    let config = AppConfig::load();
    let resolved_terminal = terminal::TerminalDetector::resolve(&config.terminal);
    terminal::launch_claude(
        &resolved_terminal,
        &prompt,
        working_dir.as_deref(),
        &config.wsl_shell,
        config.wsl_directory.as_deref(),
    )
}

#[tauri::command]
async fn update_recent_directory(directory: String) -> Result<(), String> {
    let mut config = AppConfig::load();

    // Remove if already exists
    config.recent_directories.retain(|d| d != &directory);

    // Add to front
    config.recent_directories.insert(0, directory.clone());

    // Keep only last 5
    config.recent_directories.truncate(5);

    // Update last directory
    config.last_directory = Some(directory);

    config.save()
}

#[tauri::command]
async fn update_wsl_directory(directory: String) -> Result<(), String> {
    let mut config = AppConfig::load();

    // Remove if already exists
    config.wsl_recent_directories.retain(|d| d != &directory);

    // Add to front
    config.wsl_recent_directories.insert(0, directory.clone());

    // Keep only last 5
    config.wsl_recent_directories.truncate(5);

    // Update wsl directory
    config.wsl_directory = Some(directory);

    config.save()
}

#[tauri::command]
async fn hide_window(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_wsl_root_path() -> Result<String, String> {
    // Get the default WSL distribution name
    let output = std::process::Command::new("wsl")
        .args(["-l", "-q"])
        .output()
        .map_err(|e| format!("Failed to run wsl command: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get WSL distributions".to_string());
    }

    // wsl -l -q outputs UTF-16LE on Windows
    let stdout = output.stdout;
    let decoded = if stdout.len() >= 2 && stdout[0] == 0xFF && stdout[1] == 0xFE {
        // Skip BOM
        String::from_utf16_lossy(
            &stdout[2..]
                .chunks(2)
                .filter_map(|chunk| {
                    if chunk.len() == 2 {
                        Some(u16::from_le_bytes([chunk[0], chunk[1]]))
                    } else {
                        None
                    }
                })
                .collect::<Vec<u16>>(),
        )
    } else {
        // Try UTF-16LE without BOM
        String::from_utf16_lossy(
            &stdout
                .chunks(2)
                .filter_map(|chunk| {
                    if chunk.len() == 2 {
                        Some(u16::from_le_bytes([chunk[0], chunk[1]]))
                    } else {
                        None
                    }
                })
                .collect::<Vec<u16>>(),
        )
    };

    // Get the first non-empty line (default distribution)
    let distro = decoded
        .lines()
        .map(|s| s.trim().trim_matches('\0'))
        .find(|s| !s.is_empty())
        .ok_or_else(|| "No WSL distribution found".to_string())?;

    Ok(format!("\\\\wsl.localhost\\{}", distro))
}

#[tauri::command]
fn unc_to_wsl_path(unc_path: String) -> Result<String, String> {
    // Handle both \\wsl.localhost\Distro\... and \\wsl$\Distro\...
    let path = unc_path.replace('/', "\\");

    let stripped = if let Some(rest) = path.strip_prefix("\\\\wsl.localhost\\") {
        rest
    } else if let Some(rest) = path.strip_prefix("\\\\wsl$\\") {
        rest
    } else {
        return Err(format!("Not a valid WSL UNC path: {}", unc_path));
    };

    // Find the first backslash after the distro name
    if let Some(pos) = stripped.find('\\') {
        let wsl_path = stripped[pos..].replace('\\', "/");
        Ok(wsl_path)
    } else {
        // Just the distro name, return root
        Ok("/".to_string())
    }
}

fn parse_shortcut(shortcut_str: &str) -> Option<Shortcut> {
    let parts: Vec<&str> = shortcut_str.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "super" | "win" | "cmd" | "meta" => modifiers |= Modifiers::SUPER,
            key => {
                key_code = match key {
                    "space" => Some(Code::Space),
                    "enter" => Some(Code::Enter),
                    "tab" => Some(Code::Tab),
                    "escape" | "esc" => Some(Code::Escape),
                    "backspace" => Some(Code::Backspace),
                    "delete" => Some(Code::Delete),
                    "insert" => Some(Code::Insert),
                    "home" => Some(Code::Home),
                    "end" => Some(Code::End),
                    "pageup" => Some(Code::PageUp),
                    "pagedown" => Some(Code::PageDown),
                    "up" => Some(Code::ArrowUp),
                    "down" => Some(Code::ArrowDown),
                    "left" => Some(Code::ArrowLeft),
                    "right" => Some(Code::ArrowRight),
                    "f1" => Some(Code::F1),
                    "f2" => Some(Code::F2),
                    "f3" => Some(Code::F3),
                    "f4" => Some(Code::F4),
                    "f5" => Some(Code::F5),
                    "f6" => Some(Code::F6),
                    "f7" => Some(Code::F7),
                    "f8" => Some(Code::F8),
                    "f9" => Some(Code::F9),
                    "f10" => Some(Code::F10),
                    "f11" => Some(Code::F11),
                    "f12" => Some(Code::F12),
                    "a" => Some(Code::KeyA),
                    "b" => Some(Code::KeyB),
                    "c" => Some(Code::KeyC),
                    "d" => Some(Code::KeyD),
                    "e" => Some(Code::KeyE),
                    "f" => Some(Code::KeyF),
                    "g" => Some(Code::KeyG),
                    "h" => Some(Code::KeyH),
                    "i" => Some(Code::KeyI),
                    "j" => Some(Code::KeyJ),
                    "k" => Some(Code::KeyK),
                    "l" => Some(Code::KeyL),
                    "m" => Some(Code::KeyM),
                    "n" => Some(Code::KeyN),
                    "o" => Some(Code::KeyO),
                    "p" => Some(Code::KeyP),
                    "q" => Some(Code::KeyQ),
                    "r" => Some(Code::KeyR),
                    "s" => Some(Code::KeyS),
                    "t" => Some(Code::KeyT),
                    "u" => Some(Code::KeyU),
                    "v" => Some(Code::KeyV),
                    "w" => Some(Code::KeyW),
                    "x" => Some(Code::KeyX),
                    "y" => Some(Code::KeyY),
                    "z" => Some(Code::KeyZ),
                    "0" => Some(Code::Digit0),
                    "1" => Some(Code::Digit1),
                    "2" => Some(Code::Digit2),
                    "3" => Some(Code::Digit3),
                    "4" => Some(Code::Digit4),
                    "5" => Some(Code::Digit5),
                    "6" => Some(Code::Digit6),
                    "7" => Some(Code::Digit7),
                    "8" => Some(Code::Digit8),
                    "9" => Some(Code::Digit9),
                    _ => None,
                };
            }
        }
    }

    key_code.map(|code| Shortcut::new(Some(modifiers), code))
}

fn show_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn show_settings_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let config = AppConfig::load();
                        if let Some(expected_shortcut) = parse_shortcut(&config.shortcut) {
                            if shortcut == &expected_shortcut {
                                toggle_main_window(app);
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Create tray menu
            let show_input =
                MenuItem::with_id(app, "show_input", "Show Input", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_input, &settings, &quit])?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show_input" => {
                        show_main_window(app);
                    }
                    "settings" => {
                        show_settings_window(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        toggle_main_window(app);
                    }
                })
                .build(app)?;

            // Register global shortcut
            let config = AppConfig::load();
            if let Some(shortcut) = parse_shortcut(&config.shortcut) {
                let _ = app.global_shortcut().register(shortcut);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_available_terminals,
            open_claude_interactive,
            hide_window,
            update_recent_directory,
            update_wsl_directory,
            get_wsl_root_path,
            unc_to_wsl_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
