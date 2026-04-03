import { describe, it, expect } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "../App";
import type { AppConfig } from "../types";
import { commandMocks } from "./setup";

// mock reset is handled in setup.ts

describe("App", () => {
  it("初期描画でgetConfigが呼ばれ、プロンプト入力とディレクトリボタンが表示される", async () => {
    render(<App />);

    await waitFor(() => {
      expect(commandMocks.getConfig).toHaveBeenCalled();
    });

    expect(screen.getByPlaceholderText("Ask Claude...")).toBeInTheDocument();
    expect(screen.getByText("(No directory selected)")).toBeInTheDocument();
  });

  it("configのlastDirectoryが反映される", async () => {
    commandMocks.getConfig.mockResolvedValue({
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
      timeoutSecs: 3600,
      enableOnStartup: false,
      enableContextMenu: false,
    } as AppConfig);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("C:\\project")).toBeInTheDocument();
    });
  });

  it("プロンプト入力→EnterでopenClaudeInteractiveとhideWindowが呼ばれる", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(commandMocks.getConfig).toHaveBeenCalled();
    });

    const input = screen.getByPlaceholderText("Ask Claude...");
    await user.type(input, "hello world");
    await user.keyboard("{Enter}");

    await waitFor(() => {
      expect(commandMocks.openClaudeInteractive).toHaveBeenCalledWith("hello world", null);
      expect(commandMocks.hideWindow).toHaveBeenCalled();
    });
  });

  it("空プロンプトではEnterしても送信されない", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(commandMocks.getConfig).toHaveBeenCalled();
    });

    commandMocks.openClaudeInteractive.mockClear();

    const input = screen.getByPlaceholderText("Ask Claude...");
    await user.click(input);
    await user.keyboard("{Enter}");

    expect(commandMocks.openClaudeInteractive).not.toHaveBeenCalled();
  });

  it("EscapeキーでhideWindowが呼ばれる", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(commandMocks.getConfig).toHaveBeenCalled();
    });

    const input = screen.getByPlaceholderText("Ask Claude...");
    await user.click(input);
    await user.keyboard("{Escape}");

    await waitFor(() => {
      expect(commandMocks.hideWindow).toHaveBeenCalled();
    });
  });

  it("ディレクトリドロップダウンが開閉する", async () => {
    const user = userEvent.setup();

    commandMocks.getConfig.mockResolvedValue({
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
      timeoutSecs: 3600,
      enableOnStartup: false,
      enableContextMenu: false,
    } as AppConfig);

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

  it("Ctrl+Enterで改行が入力され、送信されない", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(commandMocks.getConfig).toHaveBeenCalled();
    });

    commandMocks.openClaudeInteractive.mockClear();

    const textarea = screen.getByPlaceholderText("Ask Claude...");
    await user.type(textarea, "line1");
    await user.keyboard("{Control>}{Enter}{/Control}");
    await user.type(textarea, "line2");

    expect(commandMocks.openClaudeInteractive).not.toHaveBeenCalled();
    expect(textarea).toHaveValue("line1\nline2");
  });

  it("送信中にEnterを連打しても二重送信されない", async () => {
    const user = userEvent.setup();

    let resolveSubmit: () => void;
    commandMocks.openClaudeInteractive.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveSubmit = resolve;
        }),
    );

    render(<App />);

    await waitFor(() => {
      expect(commandMocks.getConfig).toHaveBeenCalled();
    });

    const textarea = screen.getByPlaceholderText("Ask Claude...");
    await user.type(textarea, "hello");

    // 1回目のEnter（送信開始・openClaudeInteractiveが未解決のまま）
    await user.keyboard("{Enter}");
    // 2回目のEnter（送信中のため無視されるべき）
    await user.keyboard("{Enter}");

    // 1回目の送信を完了させる
    resolveSubmit!();

    await waitFor(() => {
      expect(commandMocks.openClaudeInteractive).toHaveBeenCalledTimes(1);
    });
  });

  it("recentDirectories選択でcurrentDirectoryが更新される", async () => {
    const user = userEvent.setup();

    commandMocks.getConfig.mockResolvedValue({
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
      timeoutSecs: 3600,
      enableOnStartup: false,
      enableContextMenu: false,
    } as AppConfig);

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
