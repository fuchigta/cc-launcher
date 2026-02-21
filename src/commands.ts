import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  ExecutionLog,
  PluginConfig,
  PluginStatus,
  ScheduleConfig,
  SubscriptionConfig,
  TerminalInfo,
} from "./types";

// --- Config ---
export const getConfig = () => invoke<AppConfig>("get_config");
export const saveConfig = (newConfig: AppConfig) => invoke("save_config", { newConfig });
export const getAvailableTerminals = () => invoke<TerminalInfo[]>("get_available_terminals");

// --- Directory ---
export const updateRecentDirectory = (directory: string) =>
  invoke("update_recent_directory", { directory });
export const updateWslDirectory = (directory: string) =>
  invoke("update_wsl_directory", { directory });
export const getWslRootPath = () => invoke<string>("get_wsl_root_path");
export const uncToWslPath = (uncPath: string) => invoke<string>("unc_to_wsl_path", { uncPath });

// --- Window ---
export const hideWindow = () => invoke("hide_window");
export const openClaudeInteractive = (prompt: string, workingDir: string | null) =>
  invoke("open_claude_interactive", { prompt, workingDir });

// --- Logs ---
export const getLogs = (limit: number, offset: number) =>
  invoke<ExecutionLog[]>("get_logs", { limit, offset });
export const clearLogs = () => invoke("clear_logs");

// --- Schedules ---
export const getSchedules = () => invoke<ScheduleConfig[]>("get_schedules");
export const saveSchedule = (schedule: ScheduleConfig) => invoke("save_schedule", { schedule });
export const deleteSchedule = (id: string) => invoke("delete_schedule", { id });
export const toggleSchedule = (id: string, enabled: boolean) =>
  invoke("toggle_schedule", { id, enabled });
export const testRunSchedule = (id: string) => invoke<string>("test_run_schedule", { id });

// --- Plugins ---
export const getPlugins = () => invoke<PluginConfig[]>("get_plugins");
export const savePlugin = (plugin: PluginConfig) => invoke("save_plugin", { plugin });
export const deletePlugin = (id: string) => invoke("delete_plugin", { id });
export const togglePlugin = (id: string, enabled: boolean) =>
  invoke("toggle_plugin", { id, enabled });
export const getPluginStatuses = () => invoke<PluginStatus[]>("get_plugin_statuses");
export const restartPlugin = (id: string) => invoke("restart_plugin", { id });

// --- Subscriptions ---
export const getSubscriptions = () => invoke<SubscriptionConfig[]>("get_subscriptions");
export const saveSubscription = (subscription: SubscriptionConfig) =>
  invoke("save_subscription", { subscription });
export const deleteSubscription = (id: string) => invoke("delete_subscription", { id });
export const toggleSubscription = (id: string, enabled: boolean) =>
  invoke("toggle_subscription", { id, enabled });
