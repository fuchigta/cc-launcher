import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SubscriptionConfig } from "../types";
import SubscriptionForm from "./SubscriptionForm";

function SubscriptionsTab() {
  const [subscriptions, setSubscriptions] = useState<SubscriptionConfig[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingSub, setEditingSub] = useState<SubscriptionConfig | null>(null);

  const loadSubscriptions = async () => {
    try {
      const data = await invoke<SubscriptionConfig[]>("get_subscriptions");
      setSubscriptions(data);
    } catch (e) {
      console.error("Failed to load subscriptions:", e);
    }
  };

  useEffect(() => {
    loadSubscriptions();
  }, []);

  const handleSave = async (subscription: SubscriptionConfig) => {
    try {
      await invoke("save_subscription", { subscription });
      setShowForm(false);
      setEditingSub(null);
      await loadSubscriptions();
    } catch (e) {
      console.error("Failed to save subscription:", e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke("delete_subscription", { id });
      await loadSubscriptions();
    } catch (e) {
      console.error("Failed to delete subscription:", e);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await invoke("toggle_subscription", { id, enabled });
      await loadSubscriptions();
    } catch (e) {
      console.error("Failed to toggle subscription:", e);
    }
  };

  const handleEdit = (sub: SubscriptionConfig) => {
    setEditingSub(sub);
    setShowForm(true);
  };

  const handleNew = () => {
    setEditingSub(null);
    setShowForm(true);
  };

  return (
    <div>
      <div className="toolbar">
        <span>{subscriptions.length} subscription(s)</span>
        <div className="toolbar-actions">
          <button className="btn btn-primary" onClick={handleNew}>
            + New Subscription
          </button>
        </div>
      </div>

      {subscriptions.length === 0 ? (
        <div className="empty-state">
          <p>No subscriptions configured</p>
          <button className="btn btn-primary" onClick={handleNew}>
            Create your first subscription
          </button>
        </div>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Enabled</th>
              <th>Name</th>
              <th>Plugin</th>
              <th>Event</th>
              <th>Prompt</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {subscriptions.map((s) => (
              <tr key={s.id}>
                <td>
                  <button
                    className={`toggle ${s.enabled ? "active" : ""}`}
                    onClick={() => handleToggle(s.id, !s.enabled)}
                  />
                </td>
                <td>{s.name}</td>
                <td>{s.pluginName}</td>
                <td>{s.eventType}</td>
                <td
                  style={{
                    maxWidth: 200,
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {s.promptTemplate}
                </td>
                <td>
                  <button className="btn btn-sm btn-secondary" onClick={() => handleEdit(s)}>
                    Edit
                  </button>{" "}
                  <button className="btn btn-sm btn-danger" onClick={() => handleDelete(s.id)}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {showForm && (
        <SubscriptionForm
          subscription={editingSub}
          onSave={handleSave}
          onCancel={() => {
            setShowForm(false);
            setEditingSub(null);
          }}
        />
      )}
    </div>
  );
}

export default SubscriptionsTab;
