export interface AppConfig {
  shortcut: string;
  terminal: TerminalType;
  wslShell: WslShell;
  lastDirectory: string | null;
  recentDirectories: string[];
  wslDirectory: string | null;
  wslRecentDirectories: string[];
  schedules: ScheduleConfig[];
  plugins: PluginConfig[];
  subscriptions: SubscriptionConfig[];
}

export type TerminalType = "Auto" | "Pwsh" | "PowerShell" | "Cmd" | "Wsl";

export type WslShell = "Bash" | "Zsh" | "Sh";

export interface TerminalInfo {
  terminal_type: TerminalType;
  display_name: string;
  available: boolean;
}

// --- Execution ---

export type ExecutionSource =
  | { type: "Schedule"; id: string; name: string }
  | { type: "Plugin"; pluginName: string; eventType: string }
  | { type: "Manual" };

export type ExecutionStatus = "Running" | "Success" | "Failed";

export interface ExecutionLog {
  id: string;
  source: ExecutionSource;
  prompt: string;
  workingDir: string | null;
  claudeArgs: string[];
  status: ExecutionStatus;
  stdout: string;
  stderr: string;
  exitCode: number | null;
  startedAt: string;
  completedAt: string | null;
  durationMs: number | null;
}

// --- Schedule ---

export type ScheduleExpression =
  | { type: "Cron"; expression: string }
  | { type: "Interval"; seconds: number }
  | { type: "DailyAt"; time: string };

export interface ScheduleConfig {
  id: string;
  name: string;
  expression: ScheduleExpression;
  prompt: string;
  workingDir: string | null;
  claudeArgs: string[];
  enabled: boolean;
}

// --- Plugin ---

export interface PluginConfig {
  id: string;
  name: string;
  executable: string;
  args: string[];
  enabled: boolean;
}

export interface PluginStatus {
  id: string;
  name: string;
  running: boolean;
  pid: number | null;
  lastEventAt: string | null;
  error: string | null;
}

// --- Subscription ---

export interface SubscriptionConfig {
  id: string;
  name: string;
  pluginName: string;
  eventType: string;
  promptTemplate: string;
  workingDir: string | null;
  claudeArgs: string[];
  enabled: boolean;
}
