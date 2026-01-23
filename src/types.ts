export interface AppConfig {
  shortcut: string;
  terminal: TerminalType;
}

export type TerminalType = "Auto" | "Pwsh" | "PowerShell" | "Cmd";

export interface TerminalInfo {
  terminal_type: TerminalType;
  display_name: string;
  available: boolean;
}
