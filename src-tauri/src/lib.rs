pub mod config;
pub mod error;
mod headless;
mod logs;
pub mod models;
mod plugin_host;
mod scheduler;
mod subscription;
pub(crate) mod terminal;
mod windows_util;

use std::sync::Arc;

use config::AppConfig;
use error::{AppError, AppResult};
use models::{
    ExecutionLog, ExecutionSource, PluginConfig, PluginStatus, ScheduleConfig, SubscriptionConfig,
};
use plugin_host::PluginManager;
use scheduler::SchedulerManager;
use subscription::SubscriptionEngine;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use terminal::TerminalInfo;
use tokio::sync::RwLock;

struct AppState {
    scheduler: Arc<RwLock<Option<SchedulerManager>>>,
    plugin_manager: Arc<RwLock<Option<PluginManager>>>,
    subscription_engine: Arc<RwLock<Option<SubscriptionEngine>>>,
}

// --- Headless execution commands ---

#[tauri::command]
async fn run_headless(
    app_handle: tauri::AppHandle,
    prompt: String,
    working_dir: Option<String>,
    claude_args: Option<Vec<String>>,
) -> AppResult<String> {
    let args = claude_args.unwrap_or_default();
    let log = headless::execute(
        &prompt,
        working_dir.as_deref(),
        &args,
        ExecutionSource::Manual,
        &app_handle,
    )
    .await?;
    Ok(log.id)
}

#[tauri::command]
async fn get_logs(limit: Option<usize>, offset: Option<usize>) -> AppResult<Vec<ExecutionLog>> {
    logs::list_logs(limit.unwrap_or(50), offset.unwrap_or(0))
}

#[tauri::command]
async fn get_log(id: String) -> AppResult<ExecutionLog> {
    logs::get_log(&id)
}

#[tauri::command]
async fn clear_logs() -> AppResult<()> {
    logs::clear_logs()
}

// --- Config helpers ---

fn with_config<F>(updater: F) -> AppResult<AppConfig>
where
    F: FnOnce(&mut AppConfig) -> AppResult<()>,
{
    let mut config = AppConfig::load();
    updater(&mut config)?;
    config.save()?;
    Ok(config)
}

async fn reload_scheduler(app: &tauri::AppHandle, config: &AppConfig) -> AppResult<()> {
    let state = app.state::<AppState>();
    let guard = state.scheduler.read().await;
    if let Some(sched) = guard.as_ref() {
        sched
            .reload_all(&config.schedules)
            .await
            .map_err(AppError::Execution)?;
    }
    Ok(())
}

async fn reload_subscriptions(app: &tauri::AppHandle, config: &AppConfig) {
    let state = app.state::<AppState>();
    let guard = state.subscription_engine.read().await;
    if let Some(engine) = guard.as_ref() {
        engine.reload(config.subscriptions.clone()).await;
    }
}

// --- Schedule commands ---

#[tauri::command]
fn get_schedules() -> Vec<ScheduleConfig> {
    AppConfig::load().schedules
}

#[tauri::command]
async fn save_schedule(app_handle: tauri::AppHandle, schedule: ScheduleConfig) -> AppResult<()> {
    let config = with_config(|c| {
        if let Some(existing) = c.schedules.iter_mut().find(|s| s.id == schedule.id) {
            *existing = schedule.clone();
        } else {
            c.schedules.push(schedule.clone());
        }
        Ok(())
    })?;
    reload_scheduler(&app_handle, &config).await
}

#[tauri::command]
async fn delete_schedule(app_handle: tauri::AppHandle, id: String) -> AppResult<()> {
    let config = with_config(|c| {
        c.schedules.retain(|s| s.id != id);
        Ok(())
    })?;
    reload_scheduler(&app_handle, &config).await
}

#[tauri::command]
async fn toggle_schedule(app_handle: tauri::AppHandle, id: String, enabled: bool) -> AppResult<()> {
    let config = with_config(|c| {
        if let Some(schedule) = c.schedules.iter_mut().find(|s| s.id == id) {
            schedule.enabled = enabled;
            Ok(())
        } else {
            Err(AppError::NotFound("Schedule not found".to_string()))
        }
    })?;
    reload_scheduler(&app_handle, &config).await
}

#[tauri::command]
async fn test_run_schedule(app_handle: tauri::AppHandle, id: String) -> AppResult<String> {
    let config = AppConfig::load();
    let schedule = config
        .schedules
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| AppError::NotFound("Schedule not found".to_string()))?;

    let log = headless::execute(
        &schedule.prompt,
        schedule.working_dir.as_deref(),
        &schedule.claude_args,
        ExecutionSource::Schedule {
            id: schedule.id.clone(),
            name: schedule.name.clone(),
        },
        &app_handle,
    )
    .await?;
    Ok(log.id)
}

// --- Plugin commands ---

#[tauri::command]
fn get_plugins() -> Vec<PluginConfig> {
    AppConfig::load().plugins
}

#[tauri::command]
async fn save_plugin(app_handle: tauri::AppHandle, plugin: PluginConfig) -> AppResult<()> {
    let config = with_config(|c| {
        if let Some(existing) = c.plugins.iter_mut().find(|p| p.id == plugin.id) {
            *existing = plugin.clone();
        } else {
            c.plugins.push(plugin.clone());
        }
        Ok(())
    })?;

    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        pm.stop_all().await;
        pm.start_all(&config.plugins).await;
    }
    Ok(())
}

#[tauri::command]
async fn delete_plugin(app_handle: tauri::AppHandle, id: String) -> AppResult<()> {
    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        pm.stop_plugin(&id).await.ok();
    }
    drop(guard);

    with_config(|c| {
        c.plugins.retain(|p| p.id != id);
        Ok(())
    })
    .map(|_| ())
}

#[tauri::command]
async fn toggle_plugin(app_handle: tauri::AppHandle, id: String, enabled: bool) -> AppResult<()> {
    let config = with_config(|c| {
        if let Some(plugin) = c.plugins.iter_mut().find(|p| p.id == id) {
            plugin.enabled = enabled;
            Ok(())
        } else {
            Err(AppError::NotFound("Plugin not found".to_string()))
        }
    })?;

    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        if enabled {
            let plugin = config
                .plugins
                .iter()
                .find(|p| p.id == id)
                .ok_or_else(|| AppError::NotFound(format!("Plugin not found: {}", id)))?;
            if let Err(e) = pm.start_plugin(plugin).await {
                eprintln!("Failed to start plugin {}: {}", id, e);
            }
        } else {
            pm.stop_plugin(&id).await.map_err(AppError::Plugin)?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn get_plugin_statuses(app_handle: tauri::AppHandle) -> AppResult<Vec<PluginStatus>> {
    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    match guard.as_ref() {
        Some(pm) => Ok(pm.get_statuses().await),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
async fn restart_plugin(app_handle: tauri::AppHandle, id: String) -> AppResult<()> {
    let config = AppConfig::load();
    let plugin = config
        .plugins
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound("Plugin not found".to_string()))?
        .clone();

    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        pm.stop_plugin(&id).await.ok();
        pm.start_plugin(&plugin).await.map_err(AppError::Plugin)?;
    }
    Ok(())
}

// --- Subscription commands ---

#[tauri::command]
fn get_subscriptions() -> Vec<SubscriptionConfig> {
    AppConfig::load().subscriptions
}

#[tauri::command]
async fn save_subscription(
    app_handle: tauri::AppHandle,
    subscription: SubscriptionConfig,
) -> AppResult<()> {
    let config = with_config(|c| {
        if let Some(existing) = c.subscriptions.iter_mut().find(|s| s.id == subscription.id) {
            *existing = subscription.clone();
        } else {
            c.subscriptions.push(subscription.clone());
        }
        Ok(())
    })?;
    reload_subscriptions(&app_handle, &config).await;
    Ok(())
}

#[tauri::command]
async fn delete_subscription(app_handle: tauri::AppHandle, id: String) -> AppResult<()> {
    let config = with_config(|c| {
        c.subscriptions.retain(|s| s.id != id);
        Ok(())
    })?;
    reload_subscriptions(&app_handle, &config).await;
    Ok(())
}

#[tauri::command]
async fn toggle_subscription(
    app_handle: tauri::AppHandle,
    id: String,
    enabled: bool,
) -> AppResult<()> {
    let config = with_config(|c| {
        if let Some(sub) = c.subscriptions.iter_mut().find(|s| s.id == id) {
            sub.enabled = enabled;
            Ok(())
        } else {
            Err(AppError::NotFound("Subscription not found".to_string()))
        }
    })?;
    reload_subscriptions(&app_handle, &config).await;
    Ok(())
}

// --- Config commands ---

#[tauri::command]
fn get_config() -> AppResult<AppConfig> {
    Ok(AppConfig::load())
}

fn parse_directory_arg(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--directory" {
            return iter.next().cloned();
        }
    }
    None
}

const STARTUP_RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const STARTUP_APP_NAME: &str = "cc-launcher";

fn sync_startup_registry(enabled: bool) {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;

    let Ok(hkcu) =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(STARTUP_RUN_KEY, KEY_SET_VALUE)
    else {
        return;
    };

    if enabled {
        if let Ok(exe) = std::env::current_exe() {
            let value = format!("\"{}\"", exe.to_string_lossy());
            let _ = hkcu.set_value(STARTUP_APP_NAME, &value.as_str());
        }
    } else {
        let _ = hkcu.delete_value(STARTUP_APP_NAME);
    }
}

const CONTEXT_MENU_KEY_DIR: &str = "Software\\Classes\\Directory\\shell\\cc-launcher";
const CONTEXT_MENU_KEY_BG: &str = "Software\\Classes\\Directory\\Background\\shell\\cc-launcher";
const CONTEXT_MENU_HANDLER_CLSID: &str = "{4CC3A7F2-1B5E-4D9A-8F6C-3E2D1A4B5C7E}";
const CONTEXT_MENU_CLSID_KEY: &str =
    "Software\\Classes\\CLSID\\{4CC3A7F2-1B5E-4D9A-8F6C-3E2D1A4B5C7E}";

fn sync_context_menu_registry(enabled: bool) {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let shell_paths = [CONTEXT_MENU_KEY_DIR, CONTEXT_MENU_KEY_BG];

    if enabled {
        let exe_path = match std::env::current_exe() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => return,
        };
        let command_value = format!("\"{}\" --directory \"%V\"", exe_path);
        let icon_value = format!("{},0", exe_path);

        for shell_path in &shell_paths {
            let Ok((shell_key, _)) = hkcu.create_subkey_with_flags(shell_path, KEY_WRITE) else {
                continue;
            };
            let _ = shell_key.set_value("", &"cc-launcherで開く");
            let _ = shell_key.set_value("Icon", &icon_value.as_str());

            let command_path = format!("{}\\command", shell_path);
            let Ok((cmd_key, _)) = hkcu.create_subkey_with_flags(&command_path, KEY_WRITE) else {
                continue;
            };
            let _ = cmd_key.set_value("", &command_value.as_str());
        }

        // Register COM handler for Windows 11 modern context menu.
        // The DLL lives next to the exe; skip if not found (e.g. dev builds).
        let exe_dir = std::path::Path::new(&exe_path)
            .parent()
            .map(|p| p.to_path_buf());
        if let Some(dir) = exe_dir {
            let dll_path = dir.join("context_menu_handler.dll");
            if dll_path.exists() {
                let dll_str = dll_path.to_string_lossy().to_string();
                let inproc_key = format!("{}\\InprocServer32", CONTEXT_MENU_CLSID_KEY);
                if let Ok((clsid_key, _)) =
                    hkcu.create_subkey_with_flags(CONTEXT_MENU_CLSID_KEY, KEY_WRITE)
                {
                    let _ = clsid_key.set_value("", &"cc-launcher Context Menu Handler");
                }
                if let Ok((inproc, _)) = hkcu.create_subkey_with_flags(&inproc_key, KEY_WRITE) {
                    let _ = inproc.set_value("", &dll_str.as_str());
                    let _ = inproc.set_value("ThreadingModel", &"Apartment");
                }
                for shell_path in &shell_paths {
                    if let Ok(key) = hkcu.open_subkey_with_flags(shell_path, KEY_WRITE) {
                        let _ =
                            key.set_value("ExplorerCommandHandler", &CONTEXT_MENU_HANDLER_CLSID);
                    }
                }
            }
        }
    } else {
        for shell_path in &shell_paths {
            let _ = hkcu.delete_subkey_all(shell_path);
        }
        let _ = hkcu.delete_subkey_all(CONTEXT_MENU_CLSID_KEY);
    }
}

#[tauri::command]
fn save_config(new_config: AppConfig) -> AppResult<()> {
    sync_startup_registry(new_config.enable_on_startup);
    sync_context_menu_registry(new_config.enable_context_menu);
    new_config.save()
}

#[tauri::command]
fn get_available_terminals() -> Vec<TerminalInfo> {
    terminal::TerminalDetector::detect_available()
}

#[tauri::command]
async fn open_claude_interactive(prompt: String, working_dir: Option<String>) -> AppResult<()> {
    let config = AppConfig::load();
    let resolved_terminal = terminal::TerminalDetector::resolve(&config.terminal);
    terminal::launch_claude(
        &resolved_terminal,
        &prompt,
        working_dir.as_deref(),
        &config.wsl_shell,
        config.wsl_directory.as_deref(),
    )
    .map_err(AppError::Execution)
}

#[tauri::command]
async fn resume_claude_session(session_id: String, working_dir: Option<String>) -> AppResult<()> {
    let config = AppConfig::load();
    let resolved_terminal = terminal::TerminalDetector::resolve(&config.terminal);
    terminal::resume_claude(
        &resolved_terminal,
        &session_id,
        working_dir.as_deref(),
        &config.wsl_shell,
        config.wsl_directory.as_deref(),
    )
    .map_err(AppError::Execution)
}

fn update_directory_list(list: &mut Vec<String>, last: &mut Option<String>, directory: String) {
    list.retain(|d| d != &directory);
    list.insert(0, directory.clone());
    list.truncate(5);
    *last = Some(directory);
}

#[tauri::command]
async fn update_recent_directory(directory: String) -> AppResult<()> {
    with_config(|c| {
        update_directory_list(&mut c.recent_directories, &mut c.last_directory, directory);
        Ok(())
    })
    .map(|_| ())
}

#[tauri::command]
async fn update_wsl_directory(directory: String) -> AppResult<()> {
    with_config(|c| {
        update_directory_list(
            &mut c.wsl_recent_directories,
            &mut c.wsl_directory,
            directory,
        );
        Ok(())
    })
    .map(|_| ())
}

#[tauri::command]
async fn hide_window(window: tauri::Window) -> AppResult<()> {
    window
        .hide()
        .map_err(|e| AppError::Execution(e.to_string()))
}

#[tauri::command]
fn get_wsl_root_path() -> AppResult<String> {
    let output = windows_util::no_window_command("wsl")
        .args(["-l", "-q"])
        .output()
        .map_err(|e| AppError::Execution(format!("Failed to run wsl command: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::Execution(
            "Failed to get WSL distributions".to_string(),
        ));
    }

    let decoded = decode_utf16le(&output.stdout);
    let distro = decoded
        .lines()
        .map(|s| s.trim().trim_matches('\0'))
        .find(|s| !s.is_empty())
        .ok_or_else(|| AppError::NotFound("No WSL distribution found".to_string()))?;

    Ok(format!("\\\\wsl.localhost\\{}", distro))
}

#[tauri::command]
fn unc_to_wsl_path(unc_path: String) -> AppResult<String> {
    let path = unc_path.replace('/', "\\");

    let stripped = if let Some(rest) = path.strip_prefix("\\\\wsl.localhost\\") {
        rest
    } else if let Some(rest) = path.strip_prefix("\\\\wsl$\\") {
        rest
    } else {
        return Err(AppError::Execution(format!(
            "Not a valid WSL UNC path: {}",
            unc_path
        )));
    };

    if let Some(pos) = stripped.find('\\') {
        let wsl_path = stripped[pos..].replace('\\', "/");
        Ok(wsl_path)
    } else {
        Ok("/".to_string())
    }
}

fn decode_utf16le(bytes: &[u8]) -> String {
    let start = if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        2
    } else {
        0
    };
    String::from_utf16_lossy(
        &bytes[start..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect::<Vec<_>>(),
    )
}

fn parse_key_code(key: &str) -> Option<Code> {
    const KEY_MAP: &[(&str, Code)] = &[
        ("space", Code::Space),
        ("enter", Code::Enter),
        ("tab", Code::Tab),
        ("escape", Code::Escape),
        ("esc", Code::Escape),
        ("backspace", Code::Backspace),
        ("delete", Code::Delete),
        ("insert", Code::Insert),
        ("home", Code::Home),
        ("end", Code::End),
        ("pageup", Code::PageUp),
        ("pagedown", Code::PageDown),
        ("up", Code::ArrowUp),
        ("down", Code::ArrowDown),
        ("left", Code::ArrowLeft),
        ("right", Code::ArrowRight),
        ("f1", Code::F1),
        ("f2", Code::F2),
        ("f3", Code::F3),
        ("f4", Code::F4),
        ("f5", Code::F5),
        ("f6", Code::F6),
        ("f7", Code::F7),
        ("f8", Code::F8),
        ("f9", Code::F9),
        ("f10", Code::F10),
        ("f11", Code::F11),
        ("f12", Code::F12),
        ("a", Code::KeyA),
        ("b", Code::KeyB),
        ("c", Code::KeyC),
        ("d", Code::KeyD),
        ("e", Code::KeyE),
        ("f", Code::KeyF),
        ("g", Code::KeyG),
        ("h", Code::KeyH),
        ("i", Code::KeyI),
        ("j", Code::KeyJ),
        ("k", Code::KeyK),
        ("l", Code::KeyL),
        ("m", Code::KeyM),
        ("n", Code::KeyN),
        ("o", Code::KeyO),
        ("p", Code::KeyP),
        ("q", Code::KeyQ),
        ("r", Code::KeyR),
        ("s", Code::KeyS),
        ("t", Code::KeyT),
        ("u", Code::KeyU),
        ("v", Code::KeyV),
        ("w", Code::KeyW),
        ("x", Code::KeyX),
        ("y", Code::KeyY),
        ("z", Code::KeyZ),
        ("0", Code::Digit0),
        ("1", Code::Digit1),
        ("2", Code::Digit2),
        ("3", Code::Digit3),
        ("4", Code::Digit4),
        ("5", Code::Digit5),
        ("6", Code::Digit6),
        ("7", Code::Digit7),
        ("8", Code::Digit8),
        ("9", Code::Digit9),
    ];
    KEY_MAP.iter().find(|(k, _)| *k == key).map(|(_, c)| *c)
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
            key => key_code = parse_key_code(key),
        }
    }

    key_code.map(|code| Shortcut::new(Some(modifiers), code))
}

fn center_on_cursor_monitor<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    if let Some((mon_x, mon_y, mon_w, mon_h)) = windows_util::get_cursor_monitor_work_area() {
        if let Ok(size) = window.outer_size() {
            let win_w = size.width as i32;
            let win_h = size.height as i32;
            let x = mon_x + (mon_w - win_w) / 2;
            let y = mon_y + (mon_h - win_h) / 2;
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
            return;
        }
    }
    let _ = window.center();
}

fn show_window<R: Runtime>(app: &tauri::AppHandle<R>, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        if label == "main" {
            center_on_cursor_monitor(&window);
        } else {
            let _ = window.center();
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            center_on_cursor_monitor(&window);
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(dir) = parse_directory_arg(&args) {
                let _ = app.emit("set-directory", &dir);
                show_window(app, "main");
            } else {
                toggle_main_window(app);
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
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
        .manage(AppState {
            scheduler: Arc::new(RwLock::new(None)),
            plugin_manager: Arc::new(RwLock::new(None)),
            subscription_engine: Arc::new(RwLock::new(None)),
        })
        .setup(|app| {
            // Create tray menu
            let show_input =
                MenuItem::with_id(app, "show_input", "Show Input", true, None::<&str>)?;
            let manager = MenuItem::with_id(app, "manager", "Manager", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_input, &manager, &settings, &quit])?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("cc-launcher")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show_input" => {
                        show_window(app, "main");
                    }
                    "manager" => {
                        show_window(app, "manager");
                    }
                    "settings" => {
                        show_window(app, "settings");
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
            sync_startup_registry(config.enable_on_startup);
            sync_context_menu_registry(config.enable_context_menu);

            // Handle --directory arg on first launch (no existing instance)
            let args: Vec<String> = std::env::args().collect();
            if let Some(dir) = parse_directory_arg(&args) {
                let app_handle_dir = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    let _ = app_handle_dir.emit("set-directory", &dir);
                    show_window(&app_handle_dir, "main");
                });
            }

            if let Some(shortcut) = parse_shortcut(&config.shortcut) {
                let _ = app.global_shortcut().register(shortcut);
            }

            // Initialize scheduler
            let app_handle = app.handle().clone();
            let schedules = config.schedules.clone();
            tauri::async_runtime::spawn(async move {
                match SchedulerManager::new(app_handle.clone()).await {
                    Ok(sched) => {
                        if let Err(e) = sched.reload_all(&schedules).await {
                            eprintln!("Failed to load schedules: {}", e);
                        }
                        let state = app_handle.state::<AppState>();
                        let mut guard = state.scheduler.write().await;
                        *guard = Some(sched);
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize scheduler: {}", e);
                    }
                }
            });

            // Initialize subscription engine + plugin manager
            let app_handle2 = app.handle().clone();
            let plugins = config.plugins.clone();
            let subscriptions = config.subscriptions.clone();
            tauri::async_runtime::spawn(async move {
                let engine = SubscriptionEngine::new(subscriptions);

                {
                    let state = app_handle2.state::<AppState>();
                    let mut guard = state.subscription_engine.write().await;
                    *guard = Some(engine);
                }

                let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);
                let pm = PluginManager::new(event_tx, app_handle2.clone());
                pm.start_all(&plugins).await;

                {
                    let state = app_handle2.state::<AppState>();
                    let mut guard = state.plugin_manager.write().await;
                    *guard = Some(pm);
                }

                // Event loop: route plugin events to subscription engine
                let app_for_events = app_handle2.clone();
                while let Some((plugin_name, event)) = event_rx.recv().await {
                    let state = app_for_events.state::<AppState>();
                    let guard = state.subscription_engine.read().await;
                    if let Some(engine) = guard.as_ref() {
                        engine
                            .process_event(&plugin_name, &event, &app_for_events)
                            .await;
                    }
                }
            });

            // File watcher: reload config when config.json is modified externally (e.g. by CLI)
            let app_handle_watcher = app.handle().clone();
            let config_path_watch = AppConfig::config_path();
            let config_dir = config_path_watch
                .parent()
                .unwrap_or(&config_path_watch)
                .to_path_buf();

            std::thread::spawn(move || {
                use notify::{RecursiveMode, Watcher};
                use std::sync::mpsc;

                let (tx, rx) = mpsc::sync_channel::<()>(1);
                let config_path_clone = config_path_watch.clone();

                let watcher_result =
                    notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                        if let Ok(event) = res {
                            if matches!(event.kind, notify::EventKind::Modify(_))
                                && event
                                    .paths
                                    .iter()
                                    .any(|p| p.file_name() == config_path_clone.file_name())
                            {
                                let _ = tx.try_send(());
                            }
                        }
                    });

                let mut watcher = match watcher_result {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("Failed to create file watcher: {}", e);
                        return;
                    }
                };

                if let Err(e) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
                    eprintln!("Failed to watch config dir: {}", e);
                    return;
                }

                while rx.recv().is_ok() {
                    // Debounce: drain extra events
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    while rx.try_recv().is_ok() {}

                    let config = AppConfig::load();
                    let app_clone = app_handle_watcher.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = reload_scheduler(&app_clone, &config).await {
                            eprintln!("Config watcher: failed to reload scheduler: {}", e);
                        }
                        reload_subscriptions(&app_clone, &config).await;
                        let state = app_clone.state::<AppState>();
                        let guard = state.plugin_manager.read().await;
                        if let Some(pm) = guard.as_ref() {
                            pm.stop_all().await;
                            pm.start_all(&config.plugins).await;
                        }
                    });
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label();
                if label == "settings" || label == "manager" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_available_terminals,
            open_claude_interactive,
            resume_claude_session,
            hide_window,
            update_recent_directory,
            update_wsl_directory,
            get_wsl_root_path,
            unc_to_wsl_path,
            run_headless,
            get_logs,
            get_log,
            clear_logs,
            get_schedules,
            save_schedule,
            delete_schedule,
            toggle_schedule,
            test_run_schedule,
            get_plugins,
            save_plugin,
            delete_plugin,
            toggle_plugin,
            get_plugin_statuses,
            restart_plugin,
            get_subscriptions,
            save_subscription,
            delete_subscription,
            toggle_subscription
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_directory_arg_found() {
        let args = vec![
            "cc-launcher.exe".to_string(),
            "--directory".to_string(),
            "C:\\project".to_string(),
        ];
        assert_eq!(parse_directory_arg(&args), Some("C:\\project".to_string()));
    }

    #[test]
    fn parse_directory_arg_not_found() {
        let args = vec!["cc-launcher.exe".to_string()];
        assert_eq!(parse_directory_arg(&args), None);
    }

    #[test]
    fn parse_directory_arg_no_value() {
        let args = vec!["cc-launcher.exe".to_string(), "--directory".to_string()];
        assert_eq!(parse_directory_arg(&args), None);
    }

    #[test]
    fn unc_to_wsl_path_localhost() {
        let result = unc_to_wsl_path("\\\\wsl.localhost\\Ubuntu\\home\\user".to_string());
        assert_eq!(result.unwrap(), "/home/user");
    }

    #[test]
    fn unc_to_wsl_path_wsl_dollar() {
        let result = unc_to_wsl_path("\\\\wsl$\\Ubuntu\\home\\user\\project".to_string());
        assert_eq!(result.unwrap(), "/home/user/project");
    }

    #[test]
    fn unc_to_wsl_path_root() {
        let result = unc_to_wsl_path("\\\\wsl.localhost\\Ubuntu".to_string());
        assert_eq!(result.unwrap(), "/");
    }

    #[test]
    fn unc_to_wsl_path_invalid() {
        let result = unc_to_wsl_path("C:\\Users\\test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn unc_to_wsl_path_forward_slashes() {
        let result = unc_to_wsl_path("//wsl.localhost/Ubuntu/home/user".to_string());
        assert_eq!(result.unwrap(), "/home/user");
    }
}
