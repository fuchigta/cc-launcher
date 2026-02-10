import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
// getCurrentWindow mock is a singleton defined in setup.ts
import Settings from "../Settings";
import type { AppConfig } from "../types";

const mockedInvoke = vi.mocked(invoke);

// mock reset is handled in setup.ts

describe("Settings", () => {
  it("config読み込み前にLoading...が表示される", () => {
    // Make invoke hang so config never loads
    mockedInvoke.mockImplementation(() => new Promise(() => {}));
    render(<Settings />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("config読み込み後にフォームが描画される", async () => {
    render(<Settings />);

    await waitFor(() => {
      expect(screen.getByText("Settings")).toBeInTheDocument();
      expect(screen.getByText("Save")).toBeInTheDocument();
      expect(screen.getByText("Close")).toBeInTheDocument();
    });

    expect(screen.getByDisplayValue("Ctrl+Shift+Space")).toBeInTheDocument();
  });

  it("get_available_terminalsの結果がselectに反映される", async () => {
    render(<Settings />);

    await waitFor(() => {
      expect(screen.getByText("Save")).toBeInTheDocument();
    });

    const select = screen.getByRole("combobox");
    const options = select.querySelectorAll("option");
    const optionTexts = Array.from(options).map((o) => o.textContent);

    expect(optionTexts.some((t) => t?.includes("PowerShell 7"))).toBe(true);
    expect(optionTexts.some((t) => t?.includes("Windows PowerShell"))).toBe(true);
    expect(optionTexts.some((t) => t?.includes("Command Prompt"))).toBe(true);
  });

  it("Recordボタンクリックでショートカット記録が開始される", async () => {
    const user = userEvent.setup();
    render(<Settings />);

    await waitFor(() => {
      expect(screen.getByText("Record")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Record"));
    expect(screen.getByText("Recording...")).toBeInTheDocument();
  });

  it("Save実行でsave_configがinvokeされる", async () => {
    const user = userEvent.setup();
    render(<Settings />);

    await waitFor(() => {
      expect(screen.getByText("Save")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith(
        "save_config",
        expect.objectContaining({
          newConfig: expect.objectContaining({
            shortcut: "Ctrl+Shift+Space",
            terminal: "Auto",
          }),
        }),
      );
    });
  });

  it("Close実行でwindow.hide()が呼ばれる", async () => {
    const user = userEvent.setup();
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const mockHide = getCurrentWindow().hide as ReturnType<typeof vi.fn>;
    mockHide.mockClear();

    render(<Settings />);

    await waitFor(() => {
      expect(screen.getByText("Close")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Close"));

    expect(mockHide).toHaveBeenCalled();
  });

  it("terminal=Wslの時にWSL Shell selectが表示される", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_config") {
        return Promise.resolve({
          shortcut: "Ctrl+Shift+Space",
          terminal: "Wsl",
          wslShell: "Bash",
          lastDirectory: null,
          recentDirectories: [],
          wslDirectory: null,
          wslRecentDirectories: [],
          schedules: [],
          plugins: [],
          subscriptions: [],
        } as AppConfig);
      }
      if (cmd === "get_available_terminals") {
        return Promise.resolve([{ terminal_type: "Wsl", display_name: "WSL", available: true }]);
      }
      return Promise.resolve(null);
    });

    render(<Settings />);

    await waitFor(() => {
      expect(screen.getByText("WSL Shell")).toBeInTheDocument();
    });
  });
});
