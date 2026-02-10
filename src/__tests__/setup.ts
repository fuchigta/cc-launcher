import { vi, afterEach, beforeEach } from "vitest";
import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import type { AppConfig } from "../types";

export const defaultInvokeHandlers: Record<string, unknown> = {
  get_config: {
    shortcut: "Ctrl+Shift+Space",
    terminal: "Auto",
    wslShell: "Bash",
    lastDirectory: null,
    recentDirectories: [],
    wslDirectory: null,
    wslRecentDirectories: [],
    schedules: [],
    plugins: [],
    subscriptions: [],
  } as AppConfig,
  get_available_terminals: [
    { terminal_type: "Pwsh", display_name: "PowerShell 7", available: true },
    { terminal_type: "PowerShell", display_name: "Windows PowerShell", available: true },
    { terminal_type: "Cmd", display_name: "Command Prompt", available: true },
    { terminal_type: "Wsl", display_name: "WSL", available: false },
  ],
  save_config: null,
  hide_window: null,
  open_claude_interactive: null,
  update_recent_directory: null,
  update_wsl_directory: null,
  get_wsl_root_path: "\\\\wsl.localhost\\Ubuntu",
  unc_to_wsl_path: "/home/user",
  get_schedules: [],
  get_plugins: [],
  get_subscriptions: [],
  get_logs: [],
  get_plugin_statuses: [],
};

let invokeRef: ReturnType<typeof vi.fn>;

vi.mock("@tauri-apps/api/core", () => {
  invokeRef = vi.fn((cmd: string, _args?: unknown) => {
    return Promise.resolve(defaultInvokeHandlers[cmd] ?? null);
  });
  return {
    invoke: invokeRef,
    transformCallback: vi.fn(),
  };
});

const mockWindow = {
  onFocusChanged: vi.fn(() => Promise.resolve(() => {})),
  isFocused: vi.fn(() => Promise.resolve(true)),
  setSize: vi.fn(),
  setAlwaysOnTop: vi.fn(),
  hide: vi.fn(),
};

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => mockWindow,
  LogicalSize: class {
    constructor(
      public width: number,
      public height: number,
    ) {}
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(() => Promise.resolve(null)),
}));

afterEach(() => {
  cleanup();
});

beforeEach(() => {
  invokeRef.mockReset();
  invokeRef.mockImplementation((cmd: string, _args?: unknown) => {
    return Promise.resolve(defaultInvokeHandlers[cmd] ?? null);
  });
});
