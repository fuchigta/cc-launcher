export interface AppConfig {
  shortcut: string;
  terminal: TerminalType;
  wslShell: WslShell;
  lastDirectory: string | null;
  recentDirectories: string[];
  wslDirectory: string | null;
  wslRecentDirectories: string[];
}

export type TerminalType = "Auto" | "Pwsh" | "PowerShell" | "Cmd" | "Wsl";

export type WslShell = "Bash" | "Zsh" | "Sh";

export interface TerminalInfo {
  terminal_type: TerminalType;
  display_name: string;
  available: boolean;
}
