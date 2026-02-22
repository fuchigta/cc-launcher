import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import type { PluginConfig, PluginStatus } from "../types";
import { useCrudTab } from "../hooks/useCrudTab";
import {
  getPlugins,
  savePlugin,
  deletePlugin,
  togglePlugin,
  getPluginStatuses,
  restartPlugin,
} from "../commands";
import PluginForm from "./PluginForm";
import CrudTabLayout from "./CrudTabLayout";

function PluginsTab() {
  const [statuses, setStatuses] = useState<PluginStatus[]>([]);

  const loadStatuses = useCallback(async () => {
    try {
      const data = await getPluginStatuses();
      setStatuses(data);
    } catch (e) {
      console.error("Failed to load plugin statuses:", e);
    }
  }, []);

  const {
    items: plugins,
    showForm,
    editingItem: editingPlugin,
    error,
    handleSave,
    handleDelete,
    handleToggle,
    handleEdit,
    handleNew,
    closeForm,
    clearError,
  } = useCrudTab<PluginConfig>(
    {
      getAll: getPlugins,
      save: savePlugin,
      delete: deletePlugin,
      toggle: togglePlugin,
    },
    loadStatuses,
  );

  useEffect(() => {
    loadStatuses();
    const unlisten = listen("plugin-status-changed", () => {
      loadStatuses();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadStatuses]);

  const getStatus = (id: string): PluginStatus | undefined => statuses.find((s) => s.id === id);

  const renderStatusBadge = (status: PluginStatus | undefined): React.ReactNode => {
    if (status?.running) {
      return <span className="badge badge-success">Running (PID: {status.pid})</span>;
    }
    if (status?.error) {
      return <span className="badge badge-error">{status.error}</span>;
    }
    return <span className="badge badge-running">Stopped</span>;
  };

  const handleRestart = async (id: string) => {
    try {
      await restartPlugin(id);
      await loadStatuses();
    } catch (e) {
      console.error("Failed to restart plugin:", e);
    }
  };

  return (
    <>
      <CrudTabLayout
        error={error}
        clearError={clearError}
        itemCount={plugins.length}
        itemLabel="plugin"
        newButtonLabel="+ New Plugin"
        emptyMessage="No plugins configured"
        emptyButtonLabel="Register your first plugin"
        onNew={handleNew}
        headers={["Enabled", "Name", "Executable", "Status", "Actions"]}
      >
        {plugins.map((p) => {
          const status = getStatus(p.id);
          return (
            <tr key={p.id}>
              <td>
                <button
                  className={`toggle ${p.enabled ? "active" : ""}`}
                  onClick={() => handleToggle(p.id, !p.enabled)}
                />
              </td>
              <td>{p.name}</td>
              <td className="truncated-cell">{p.executable}</td>
              <td>{renderStatusBadge(status)}</td>
              <td>
                <button className="btn btn-sm btn-secondary" onClick={() => handleRestart(p.id)}>
                  Restart
                </button>{" "}
                <button className="btn btn-sm btn-secondary" onClick={() => handleEdit(p)}>
                  Edit
                </button>{" "}
                <button className="btn btn-sm btn-danger" onClick={() => handleDelete(p.id)}>
                  Delete
                </button>
              </td>
            </tr>
          );
        })}
      </CrudTabLayout>

      {showForm && (
        <PluginForm
          plugin={editingPlugin}
          onSave={(plugin) => handleSave(plugin)}
          onCancel={closeForm}
        />
      )}
    </>
  );
}

export default PluginsTab;
