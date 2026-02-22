import type { SubscriptionConfig } from "../types";
import { useCrudTab } from "../hooks/useCrudTab";
import {
  getSubscriptions,
  saveSubscription,
  deleteSubscription,
  toggleSubscription,
} from "../commands";
import SubscriptionForm from "./SubscriptionForm";
import CrudTabLayout from "./CrudTabLayout";

function SubscriptionsTab() {
  const {
    items: subscriptions,
    showForm,
    editingItem: editingSub,
    error,
    handleSave,
    handleDelete,
    handleToggle,
    handleEdit,
    handleNew,
    closeForm,
    clearError,
  } = useCrudTab<SubscriptionConfig>({
    getAll: getSubscriptions,
    save: saveSubscription,
    delete: deleteSubscription,
    toggle: toggleSubscription,
  });

  return (
    <>
      <CrudTabLayout
        error={error}
        clearError={clearError}
        itemCount={subscriptions.length}
        itemLabel="subscription"
        newButtonLabel="+ New Subscription"
        emptyMessage="No subscriptions configured"
        emptyButtonLabel="Create your first subscription"
        onNew={handleNew}
        headers={["Enabled", "Name", "Plugin", "Event", "Prompt", "Actions"]}
      >
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
      </CrudTabLayout>

      {showForm && (
        <SubscriptionForm
          subscription={editingSub}
          onSave={(subscription) => handleSave(subscription)}
          onCancel={closeForm}
        />
      )}
    </>
  );
}

export default SubscriptionsTab;
