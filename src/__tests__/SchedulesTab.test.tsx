import { describe, it, expect } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import SchedulesTab from "../manager/SchedulesTab";
import { commandMocks } from "./setup";
import type { ScheduleConfig } from "../types";

describe("SchedulesTab", () => {
  it("スケジュールが0件の場合、empty-stateが表示される", async () => {
    commandMocks.getSchedules.mockResolvedValue([]);

    render(<SchedulesTab />);

    await waitFor(() => {
      expect(screen.getByText("No schedules configured")).toBeInTheDocument();
      expect(screen.getByText("Create your first schedule")).toBeInTheDocument();
    });
  });

  it("スケジュールが1件以上の場合、テーブルに表示される", async () => {
    const mockSchedule: ScheduleConfig = {
      id: "s1",
      name: "Daily Backup",
      prompt: "backup all files",
      enabled: true,
      expression: { type: "DailyAt", time: "09:00" },
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSchedules.mockResolvedValue([mockSchedule]);

    render(<SchedulesTab />);

    await waitFor(() => {
      expect(screen.getByText("Daily Backup")).toBeInTheDocument();
      expect(screen.getByText("Daily at 09:00")).toBeInTheDocument();
      expect(screen.getByText("backup all files")).toBeInTheDocument();
    });
  });

  it("Newボタンでフォームが表示される", async () => {
    const user = userEvent.setup();
    commandMocks.getSchedules.mockResolvedValue([]);

    render(<SchedulesTab />);

    await waitFor(() => {
      expect(screen.getByText("No schedules configured")).toBeInTheDocument();
    });

    const newButton = screen.getByText("Create your first schedule");
    await user.click(newButton);

    await waitFor(() => {
      expect(screen.getByText("New Schedule")).toBeInTheDocument();
    });
  });

  it("DeleteボタンでdeleteScheduleが呼ばれる", async () => {
    const user = userEvent.setup();
    const mockSchedule: ScheduleConfig = {
      id: "s1",
      name: "Daily Backup",
      prompt: "backup all files",
      enabled: true,
      expression: { type: "DailyAt", time: "09:00" },
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSchedules.mockResolvedValue([mockSchedule]);

    render(<SchedulesTab />);

    await waitFor(() => {
      expect(screen.getByText("Daily Backup")).toBeInTheDocument();
    });

    const deleteButton = screen.getByText("Delete");
    await user.click(deleteButton);

    await waitFor(() => {
      expect(commandMocks.deleteSchedule).toHaveBeenCalledWith("s1");
    });
  });

  it("トグルボタンでtoggleScheduleが呼ばれる", async () => {
    const user = userEvent.setup();
    const mockSchedule: ScheduleConfig = {
      id: "s1",
      name: "Daily Backup",
      prompt: "backup all files",
      enabled: true,
      expression: { type: "DailyAt", time: "09:00" },
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSchedules.mockResolvedValue([mockSchedule]);

    render(<SchedulesTab />);

    await waitFor(() => {
      expect(screen.getByText("Daily Backup")).toBeInTheDocument();
    });

    const toggleButton = screen
      .getAllByRole("button")
      .find((btn) => btn.className.includes("toggle"));
    expect(toggleButton).toBeDefined();
    await user.click(toggleButton!);

    await waitFor(() => {
      expect(commandMocks.toggleSchedule).toHaveBeenCalledWith("s1", false);
    });
  });

  it("Runボタンでtest_run_scheduleが呼ばれる", async () => {
    const user = userEvent.setup();
    const mockSchedule: ScheduleConfig = {
      id: "s1",
      name: "Daily Backup",
      prompt: "backup all files",
      enabled: true,
      expression: { type: "DailyAt", time: "09:00" },
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSchedules.mockResolvedValue([mockSchedule]);

    render(<SchedulesTab />);

    await waitFor(() => {
      expect(screen.getByText("Daily Backup")).toBeInTheDocument();
    });

    const runButton = screen.getByText("Run");
    await user.click(runButton);

    await waitFor(() => {
      expect(commandMocks.testRunSchedule).toHaveBeenCalledWith("s1");
    });
  });

  it("Cron式が正しく表示される", async () => {
    const mockSchedule: ScheduleConfig = {
      id: "s2",
      name: "Hourly Task",
      prompt: "check status",
      enabled: false,
      expression: { type: "Cron", expression: "0 * * * *" },
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSchedules.mockResolvedValue([mockSchedule]);

    render(<SchedulesTab />);

    await waitFor(() => {
      expect(screen.getByText("Cron: 0 * * * *")).toBeInTheDocument();
    });
  });

  it("Interval式が正しく表示される", async () => {
    const mockSchedule: ScheduleConfig = {
      id: "s3",
      name: "Periodic Task",
      prompt: "monitor",
      enabled: true,
      expression: { type: "Interval", seconds: 300 },
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSchedules.mockResolvedValue([mockSchedule]);

    render(<SchedulesTab />);

    await waitFor(() => {
      expect(screen.getByText("Every 300s")).toBeInTheDocument();
    });
  });
});
