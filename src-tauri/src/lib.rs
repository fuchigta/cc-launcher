mod config;
mod headless;
mod logs;
mod models;
mod plugin_host;
mod scheduler;
mod subscription;
mod terminal;

use std::os::windows::process::CommandExt;
use std::sync::Arc;

use config::AppConfig;
use models::{
    ExecutionLog, ExecutionSource, PluginConfig, PluginStatus, ScheduleConfig, SubscriptionConfig,
};
use plugin_host::PluginManager;
use scheduler::SchedulerManager;
use subscription::SubscriptionEngine;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
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
) -> Result<String, String> {
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
async fn get_logs(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ExecutionLog>, String> {
    logs::list_logs(limit.unwrap_or(50), offset.unwrap_or(0))
}

#[tauri::command]
async fn get_log(id: String) -> Result<ExecutionLog, String> {
    logs::get_log(&id)
}

#[tauri::command]
async fn clear_logs() -> Result<(), String> {
    logs::clear_logs()
}

// --- Schedule commands ---

#[tauri::command]
fn get_schedules() -> Vec<ScheduleConfig> {
    AppConfig::load().schedules
}

#[tauri::command]
async fn save_schedule(
    app_handle: tauri::AppHandle,
    schedule: ScheduleConfig,
) -> Result<(), String> {
    let mut config = AppConfig::load();
    if let Some(existing) = config.schedules.iter_mut().find(|s| s.id == schedule.id) {
        *existing = schedule;
    } else {
        config.schedules.push(schedule);
    }
    config.save()?;

    let state = app_handle.state::<AppState>();
    let guard = state.scheduler.read().await;
    if let Some(sched) = guard.as_ref() {
        sched.reload_all(&config.schedules).await?;
    }
    Ok(())
}

#[tauri::command]
async fn delete_schedule(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.schedules.retain(|s| s.id != id);
    config.save()?;

    let state = app_handle.state::<AppState>();
    let guard = state.scheduler.read().await;
    if let Some(sched) = guard.as_ref() {
        sched.reload_all(&config.schedules).await?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_schedule(
    app_handle: tauri::AppHandle,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut config = AppConfig::load();
    if let Some(schedule) = config.schedules.iter_mut().find(|s| s.id == id) {
        schedule.enabled = enabled;
    } else {
        return Err("Schedule not found".to_string());
    }
    config.save()?;

    let state = app_handle.state::<AppState>();
    let guard = state.scheduler.read().await;
    if let Some(sched) = guard.as_ref() {
        sched.reload_all(&config.schedules).await?;
    }
    Ok(())
}

#[tauri::command]
async fn test_run_schedule(app_handle: tauri::AppHandle, id: String) -> Result<String, String> {
    let config = AppConfig::load();
    let schedule = config
        .schedules
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| "Schedule not found".to_string())?;

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
async fn save_plugin(app_handle: tauri::AppHandle, plugin: PluginConfig) -> Result<(), String> {
    let mut config = AppConfig::load();
    if let Some(existing) = config.plugins.iter_mut().find(|p| p.id == plugin.id) {
        *existing = plugin;
    } else {
        config.plugins.push(plugin);
    }
    config.save()?;

    // Restart all plugins
    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        pm.stop_all().await;
        pm.start_all(&config.plugins).await;
    }
    Ok(())
}

#[tauri::command]
async fn delete_plugin(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        pm.stop_plugin(&id).await.ok();
    }
    drop(guard);

    let mut config = AppConfig::load();
    config.plugins.retain(|p| p.id != id);
    config.save()
}

#[tauri::command]
async fn toggle_plugin(
    app_handle: tauri::AppHandle,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut config = AppConfig::load();
    if let Some(plugin) = config.plugins.iter_mut().find(|p| p.id == id) {
        plugin.enabled = enabled;
    } else {
        return Err("Plugin not found".to_string());
    }
    config.save()?;

    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        if enabled {
            let plugin = config.plugins.iter().find(|p| p.id == id).unwrap();
            pm.start_plugin(plugin).await?;
        } else {
            pm.stop_plugin(&id).await?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn get_plugin_statuses(app_handle: tauri::AppHandle) -> Result<Vec<PluginStatus>, String> {
    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        Ok(pm.get_statuses().await)
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
async fn restart_plugin(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    let config = AppConfig::load();
    let plugin = config
        .plugins
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| "Plugin not found".to_string())?
        .clone();

    let state = app_handle.state::<AppState>();
    let guard = state.plugin_manager.read().await;
    if let Some(pm) = guard.as_ref() {
        pm.stop_plugin(&id).await.ok();
        pm.start_plugin(&plugin).await?;
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
) -> Result<(), String> {
    let mut config = AppConfig::load();
    if let Some(existing) = config
        .subscriptions
        .iter_mut()
        .find(|s| s.id == subscription.id)
    {
        *existing = subscription;
    } else {
        config.subscriptions.push(subscription);
    }
    config.save()?;

    let state = app_handle.state::<AppState>();
    let guard = state.subscription_engine.read().await;
    if let Some(engine) = guard.as_ref() {
        engine.reload(config.subscriptions).await;
    }
    Ok(())
}

#[tauri::command]
async fn delete_subscription(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.subscriptions.retain(|s| s.id != id);
    config.save()?;

    let state = app_handle.state::<AppState>();
    let guard = state.subscription_engine.read().await;
    if let Some(engine) = guard.as_ref() {
        engine.reload(config.subscriptions).await;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_subscription(
    app_handle: tauri::AppHandle,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut config = AppConfig::load();
    if let Some(sub) = config.subscriptions.iter_mut().find(|s| s.id == id) {
        sub.enabled = enabled;
    } else {
        return Err("Subscription not found".to_string());
    }
    config.save()?;

    let state = app_handle.state::<AppState>();
    let guard = state.subscription_engine.read().await;
    if let Some(engine) = guard.as_ref() {
        engine.reload(config.subscriptions).await;
    }
    Ok(())
}

// --- Config commands ---

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
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let output = std::process::Command::new("wsl")
        .args(["-l", "-q"])
        .creation_flags(CREATE_NO_WINDOW)
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

fn show_manager_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("manager") {
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
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show_input" => {
                        show_main_window(app);
                    }
                    "manager" => {
                        show_manager_window(app);
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
