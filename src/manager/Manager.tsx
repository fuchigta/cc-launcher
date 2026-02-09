import { useState } from "react";
import SchedulesTab from "./SchedulesTab";
import PluginsTab from "./PluginsTab";
import SubscriptionsTab from "./SubscriptionsTab";
import LogsTab from "./LogsTab";
import "./Manager.css";

type Tab = "schedules" | "plugins" | "subscriptions" | "logs";

function Manager() {
  const [activeTab, setActiveTab] = useState<Tab>("schedules");

  return (
    <div className="manager-container">
      <h2>Manager</h2>
      <div className="tab-nav">
        <button
          className={`tab-button ${activeTab === "schedules" ? "active" : ""}`}
          onClick={() => setActiveTab("schedules")}
        >
          Schedules
        </button>
        <button
          className={`tab-button ${activeTab === "plugins" ? "active" : ""}`}
          onClick={() => setActiveTab("plugins")}
        >
          Plugins
        </button>
        <button
          className={`tab-button ${activeTab === "subscriptions" ? "active" : ""}`}
          onClick={() => setActiveTab("subscriptions")}
        >
          Subscriptions
        </button>
        <button
          className={`tab-button ${activeTab === "logs" ? "active" : ""}`}
          onClick={() => setActiveTab("logs")}
        >
          Logs
        </button>
      </div>
      <div>
        {activeTab === "schedules" && <SchedulesTab />}
        {activeTab === "plugins" && <PluginsTab />}
        {activeTab === "subscriptions" && <SubscriptionsTab />}
        {activeTab === "logs" && <LogsTab />}
      </div>
    </div>
  );
}

export default Manager;
