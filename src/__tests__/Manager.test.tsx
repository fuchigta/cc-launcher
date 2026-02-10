import { describe, it, expect } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Manager from "../manager/Manager";

// mock reset is handled in setup.ts

describe("Manager", () => {
  it("4つのタブボタンが描画される", () => {
    render(<Manager />);

    expect(screen.getByText("Schedules")).toBeInTheDocument();
    expect(screen.getByText("Plugins")).toBeInTheDocument();
    expect(screen.getByText("Subscriptions")).toBeInTheDocument();
    expect(screen.getByText("Logs")).toBeInTheDocument();
  });

  it("デフォルトでSchedulesタブがactiveになっている", () => {
    render(<Manager />);

    const schedulesButton = screen.getByText("Schedules");
    expect(schedulesButton.className).toContain("active");
  });

  it("タブクリックで表示が切り替わる", async () => {
    const user = userEvent.setup();
    render(<Manager />);

    // Click Plugins tab
    await user.click(screen.getByText("Plugins"));

    await waitFor(() => {
      const pluginsButton = screen.getByText("Plugins");
      expect(pluginsButton.className).toContain("active");
    });

    // Click Logs tab
    await user.click(screen.getByText("Logs"));

    await waitFor(() => {
      const logsButton = screen.getByText("Logs");
      expect(logsButton.className).toContain("active");
    });
  });
});
