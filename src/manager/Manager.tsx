import { useState } from "react";
import SchedulesTab from "./SchedulesTab";
import PluginsTab from "./PluginsTab";
import SubscriptionsTab from "./SubscriptionsTab";
import LogsTab from "./LogsTab";
import "./Manager.css";

type Tab = "schedules" | "plugins" | "subscriptions" | "logs";

const tabs: { key: Tab; label: string; component: React.FC }[] = [
  { key: "schedules", label: "Schedules", component: SchedulesTab },
  { key: "plugins", label: "Plugins", component: PluginsTab },
  { key: "subscriptions", label: "Subscriptions", component: SubscriptionsTab },
  { key: "logs", label: "Logs", component: LogsTab },
];

function Manager() {
  const [activeTab, setActiveTab] = useState<Tab>("schedules");
  const ActiveComponent = tabs.find((t) => t.key === activeTab)?.component;

  return (
    <div className="manager-container">
      <h2>Manager</h2>
      <div className="tab-nav">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            className={`tab-button ${activeTab === tab.key ? "active" : ""}`}
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <div>{ActiveComponent && <ActiveComponent />}</div>
    </div>
  );
}

export default Manager;
