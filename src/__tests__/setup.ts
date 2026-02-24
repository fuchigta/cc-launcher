import { vi, afterEach, beforeEach } from "vitest";
import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import type {
  AppConfig,
  ExecutionLog,
  PluginConfig,
  PluginStatus,
  ScheduleConfig,
  SubscriptionConfig,
  TerminalInfo,
} from "../types";

const defaultConfig: AppConfig = {
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
  timeoutSecs: 3600,
};

const defaultTerminals: TerminalInfo[] = [
  { terminal_type: "Pwsh", display_name: "PowerShell 7", available: true },
  { terminal_type: "PowerShell", display_name: "Windows PowerShell", available: true },
  { terminal_type: "Cmd", display_name: "Command Prompt", available: true },
  { terminal_type: "Wsl", display_name: "WSL", available: false },
];

export const commandMocks = {
  getConfig: vi.fn(() => Promise.resolve(defaultConfig)),
  saveConfig: vi.fn(() => Promise.resolve()),
  getAvailableTerminals: vi.fn(() => Promise.resolve(defaultTerminals)),
  updateRecentDirectory: vi.fn(() => Promise.resolve()),
  updateWslDirectory: vi.fn(() => Promise.resolve()),
  getWslRootPath: vi.fn(() => Promise.resolve("\\\\wsl.localhost\\Ubuntu")),
  uncToWslPath: vi.fn(() => Promise.resolve("/home/user")),
  hideWindow: vi.fn(() => Promise.resolve()),
  openClaudeInteractive: vi.fn(() => Promise.resolve()),
  getLogs: vi.fn((): Promise<ExecutionLog[]> => Promise.resolve([])),
  clearLogs: vi.fn(() => Promise.resolve()),
  getSchedules: vi.fn((): Promise<ScheduleConfig[]> => Promise.resolve([])),
  saveSchedule: vi.fn(() => Promise.resolve()),
  deleteSchedule: vi.fn(() => Promise.resolve()),
  toggleSchedule: vi.fn(() => Promise.resolve()),
  testRunSchedule: vi.fn(() => Promise.resolve("")),
  getPlugins: vi.fn((): Promise<PluginConfig[]> => Promise.resolve([])),
  savePlugin: vi.fn(() => Promise.resolve()),
  deletePlugin: vi.fn(() => Promise.resolve()),
  togglePlugin: vi.fn(() => Promise.resolve()),
  getPluginStatuses: vi.fn((): Promise<PluginStatus[]> => Promise.resolve([])),
  restartPlugin: vi.fn(() => Promise.resolve()),
  getSubscriptions: vi.fn((): Promise<SubscriptionConfig[]> => Promise.resolve([])),
  saveSubscription: vi.fn(() => Promise.resolve()),
  deleteSubscription: vi.fn(() => Promise.resolve()),
  toggleSubscription: vi.fn(() => Promise.resolve()),
};

vi.mock("../commands", () => commandMocks);

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
  for (const mock of Object.values(commandMocks)) {
    mock.mockClear();
  }
  commandMocks.getConfig.mockImplementation(() => Promise.resolve(defaultConfig));
  commandMocks.getAvailableTerminals.mockImplementation(() => Promise.resolve(defaultTerminals));
  commandMocks.getWslRootPath.mockImplementation(() =>
    Promise.resolve("\\\\wsl.localhost\\Ubuntu"),
  );
  commandMocks.uncToWslPath.mockImplementation(() => Promise.resolve("/home/user"));
  commandMocks.getLogs.mockImplementation(() => Promise.resolve([]));
  commandMocks.getSchedules.mockImplementation(() => Promise.resolve([]));
  commandMocks.getPlugins.mockImplementation(() => Promise.resolve([]));
  commandMocks.getSubscriptions.mockImplementation(() => Promise.resolve([]));
  commandMocks.getPluginStatuses.mockImplementation(() => Promise.resolve([]));
  commandMocks.testRunSchedule.mockImplementation(() => Promise.resolve(""));
});
