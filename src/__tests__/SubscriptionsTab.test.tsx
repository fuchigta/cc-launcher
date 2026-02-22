import { describe, it, expect } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import SubscriptionsTab from "../manager/SubscriptionsTab";
import { commandMocks } from "./setup";
import type { SubscriptionConfig } from "../types";

describe("SubscriptionsTab", () => {
  it("サブスクリプションが0件の場合、empty-stateが表示される", async () => {
    commandMocks.getSubscriptions.mockResolvedValue([]);

    render(<SubscriptionsTab />);

    await waitFor(() => {
      expect(screen.getByText("No subscriptions configured")).toBeInTheDocument();
      expect(screen.getByText("Create your first subscription")).toBeInTheDocument();
    });
  });

  it("サブスクリプションが1件以上の場合、テーブルに表示される", async () => {
    const mockSubscription: SubscriptionConfig = {
      id: "sub1",
      name: "File Change Handler",
      pluginName: "folder-watcher",
      eventType: "file_changed",
      promptTemplate: "Handle file change: {{path}}",
      enabled: true,
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSubscriptions.mockResolvedValue([mockSubscription]);

    render(<SubscriptionsTab />);

    await waitFor(() => {
      expect(screen.getByText("File Change Handler")).toBeInTheDocument();
      expect(screen.getByText("folder-watcher")).toBeInTheDocument();
      expect(screen.getByText("file_changed")).toBeInTheDocument();
      expect(screen.getByText("Handle file change: {{path}}")).toBeInTheDocument();
    });
  });

  it("Newボタンでフォームが表示される", async () => {
    const user = userEvent.setup();
    commandMocks.getSubscriptions.mockResolvedValue([]);

    render(<SubscriptionsTab />);

    await waitFor(() => {
      expect(screen.getByText("No subscriptions configured")).toBeInTheDocument();
    });

    const newButton = screen.getByText("Create your first subscription");
    await user.click(newButton);

    await waitFor(() => {
      expect(screen.getByText("New Subscription")).toBeInTheDocument();
    });
  });

  it("DeleteボタンでdeleteSubscriptionが呼ばれる", async () => {
    const user = userEvent.setup();
    const mockSubscription: SubscriptionConfig = {
      id: "sub1",
      name: "File Change Handler",
      pluginName: "folder-watcher",
      eventType: "file_changed",
      promptTemplate: "Handle file change: {{path}}",
      enabled: true,
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSubscriptions.mockResolvedValue([mockSubscription]);

    render(<SubscriptionsTab />);

    await waitFor(() => {
      expect(screen.getByText("File Change Handler")).toBeInTheDocument();
    });

    const deleteButton = screen.getByText("Delete");
    await user.click(deleteButton);

    await waitFor(() => {
      expect(commandMocks.deleteSubscription).toHaveBeenCalledWith("sub1");
    });
  });

  it("トグルボタンでtoggleSubscriptionが呼ばれる", async () => {
    const user = userEvent.setup();
    const mockSubscription: SubscriptionConfig = {
      id: "sub1",
      name: "File Change Handler",
      pluginName: "folder-watcher",
      eventType: "file_changed",
      promptTemplate: "Handle file change: {{path}}",
      enabled: true,
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSubscriptions.mockResolvedValue([mockSubscription]);

    render(<SubscriptionsTab />);

    await waitFor(() => {
      expect(screen.getByText("File Change Handler")).toBeInTheDocument();
    });

    const toggleButton = screen
      .getAllByRole("button")
      .find((btn) => btn.className.includes("toggle"));
    expect(toggleButton).toBeDefined();
    await user.click(toggleButton!);

    await waitFor(() => {
      expect(commandMocks.toggleSubscription).toHaveBeenCalledWith("sub1", false);
    });
  });

  it("Editボタンでフォームが表示される", async () => {
    const user = userEvent.setup();
    const mockSubscription: SubscriptionConfig = {
      id: "sub1",
      name: "File Change Handler",
      pluginName: "folder-watcher",
      eventType: "file_changed",
      promptTemplate: "Handle file change: {{path}}",
      enabled: true,
      workingDir: null,
      claudeArgs: [],
    };

    commandMocks.getSubscriptions.mockResolvedValue([mockSubscription]);

    render(<SubscriptionsTab />);

    await waitFor(() => {
      expect(screen.getByText("File Change Handler")).toBeInTheDocument();
    });

    const editButton = screen.getByText("Edit");
    await user.click(editButton);

    await waitFor(() => {
      expect(screen.getByText("Edit Subscription")).toBeInTheDocument();
    });
  });

  it("複数のサブスクリプションが表示される", async () => {
    const mockSubscriptions: SubscriptionConfig[] = [
      {
        id: "sub1",
        name: "File Change Handler",
        pluginName: "folder-watcher",
        eventType: "file_changed",
        promptTemplate: "Handle file change: {{path}}",
        enabled: true,
        workingDir: null,
        claudeArgs: [],
      },
      {
        id: "sub2",
        name: "Git Commit Handler",
        pluginName: "git-watcher",
        eventType: "commit",
        promptTemplate: "Review commit: {{hash}}",
        enabled: false,
        workingDir: null,
        claudeArgs: [],
      },
    ];

    commandMocks.getSubscriptions.mockResolvedValue(mockSubscriptions);

    render(<SubscriptionsTab />);

    await waitFor(() => {
      expect(screen.getByText("File Change Handler")).toBeInTheDocument();
      expect(screen.getByText("Git Commit Handler")).toBeInTheDocument();
      expect(screen.getByText("folder-watcher")).toBeInTheDocument();
      expect(screen.getByText("git-watcher")).toBeInTheDocument();
    });
  });
});
