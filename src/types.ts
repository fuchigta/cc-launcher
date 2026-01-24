export interface AppConfig {
  shortcut: string;
  terminal: TerminalType;
  lastDirectory: string | null;
  recentDirectories: string[];
}

export type TerminalType = "Auto" | "Pwsh" | "PowerShell" | "Cmd";

export interface TerminalInfo {
  terminal_type: TerminalType;
  display_name: string;
  available: boolean;
}
