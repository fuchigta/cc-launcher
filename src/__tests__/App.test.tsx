import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import App from "../App";
import type { AppConfig } from "../types";

const mockedInvoke = vi.mocked(invoke);

// mock reset is handled in setup.ts

describe("App", () => {
  it("初期描画でget_configが呼ばれ、プロンプト入力とディレクトリボタンが表示される", async () => {
    render(<App />);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_config");
    });

    expect(screen.getByPlaceholderText("Ask Claude...")).toBeInTheDocument();
    expect(screen.getByText("(No directory selected)")).toBeInTheDocument();
  });

  it("configのlastDirectoryが反映される", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_config") {
        return Promise.resolve({
          shortcut: "Ctrl+Shift+Space",
          terminal: "Auto",
          wslShell: "Bash",
          lastDirectory: "C:\\project",
          recentDirectories: ["C:\\project"],
          wslDirectory: null,
          wslRecentDirectories: [],
          schedules: [],
          plugins: [],
          subscriptions: [],
        } as AppConfig);
      }
      return Promise.resolve(null);
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("C:\\project")).toBeInTheDocument();
    });
  });

  it("プロンプト入力→Enterでopen_claude_interactiveとhide_windowが呼ばれる", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_config");
    });

    const input = screen.getByPlaceholderText("Ask Claude...");
    await user.type(input, "hello world");
    await user.keyboard("{Enter}");

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("open_claude_interactive", {
        prompt: "hello world",
        workingDir: null,
      });
      expect(mockedInvoke).toHaveBeenCalledWith("hide_window");
    });
  });

  it("空プロンプトではEnterしても送信されない", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_config");
    });

    mockedInvoke.mockClear();

    const input = screen.getByPlaceholderText("Ask Claude...");
    await user.click(input);
    await user.keyboard("{Enter}");

    expect(mockedInvoke).not.toHaveBeenCalledWith("open_claude_interactive", expect.anything());
  });

  it("Escapeキーでhide_windowが呼ばれる", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_config");
    });

    const input = screen.getByPlaceholderText("Ask Claude...");
    await user.click(input);
    await user.keyboard("{Escape}");

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("hide_window");
    });
  });

  it("ディレクトリドロップダウンが開閉する", async () => {
    const user = userEvent.setup();

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_config") {
        return Promise.resolve({
          shortcut: "Ctrl+Shift+Space",
          terminal: "Auto",
          wslShell: "Bash",
          lastDirectory: "C:\\project",
          recentDirectories: ["C:\\project", "C:\\other"],
          wslDirectory: null,
          wslRecentDirectories: [],
          schedules: [],
          plugins: [],
          subscriptions: [],
        } as AppConfig);
      }
      return Promise.resolve(null);
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("C:\\project")).toBeInTheDocument();
    });

    const dirButton = screen.getByText("C:\\project").closest("button")!;
    await user.click(dirButton);

    expect(screen.getByText("Browse...")).toBeInTheDocument();
    expect(screen.getByText("C:\\other")).toBeInTheDocument();

    await user.click(dirButton);

    expect(screen.queryByText("Browse...")).not.toBeInTheDocument();
  });

  it("recentDirectories選択でcurrentDirectoryが更新される", async () => {
    const user = userEvent.setup();

    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_config") {
        return Promise.resolve({
          shortcut: "Ctrl+Shift+Space",
          terminal: "Auto",
          wslShell: "Bash",
          lastDirectory: "C:\\project",
          recentDirectories: ["C:\\project", "C:\\other"],
          wslDirectory: null,
          wslRecentDirectories: [],
          schedules: [],
          plugins: [],
          subscriptions: [],
        } as AppConfig);
      }
      return Promise.resolve(null);
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("C:\\project")).toBeInTheDocument();
    });

    const dirButton = screen.getByText("C:\\project").closest("button")!;
    await user.click(dirButton);

    const otherDir = screen.getByText("C:\\other");
    await user.click(otherDir.closest("button")!);

    // After selection, the button should show the new directory
    await waitFor(() => {
      expect(screen.getByText("C:\\other")).toBeInTheDocument();
    });
  });
});
