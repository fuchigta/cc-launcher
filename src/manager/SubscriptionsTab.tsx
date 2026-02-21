import type { SubscriptionConfig } from "../types";
import { useCrudTab } from "../hooks/useCrudTab";
import SubscriptionForm from "./SubscriptionForm";

function SubscriptionsTab() {
  const {
    items: subscriptions,
    showForm,
    editingItem: editingSub,
    handleSave,
    handleDelete,
    handleToggle,
    handleEdit,
    handleNew,
    closeForm,
  } = useCrudTab<SubscriptionConfig>({
    get: "get_subscriptions",
    save: "save_subscription",
    delete: "delete_subscription",
    toggle: "toggle_subscription",
  });

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
                <td className="truncated-cell">{s.promptTemplate}</td>
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
          onSave={(subscription) => handleSave("subscription", subscription)}
          onCancel={closeForm}
        />
      )}
    </div>
  );
}

export default SubscriptionsTab;
