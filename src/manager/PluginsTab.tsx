import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PluginConfig, PluginStatus } from "../types";
import { useCrudTab } from "../hooks/useCrudTab";
import PluginForm from "./PluginForm";

function PluginsTab() {
  const [statuses, setStatuses] = useState<PluginStatus[]>([]);

  const loadStatuses = useCallback(async () => {
    try {
      const data = await invoke<PluginStatus[]>("get_plugin_statuses");
      setStatuses(data);
    } catch (e) {
      console.error("Failed to load plugin statuses:", e);
    }
  }, []);

  const {
    items: plugins,
    showForm,
    editingItem: editingPlugin,
    handleSave,
    handleDelete,
    handleToggle,
    handleEdit,
    handleNew,
    closeForm,
  } = useCrudTab<PluginConfig>(
    {
      get: "get_plugins",
      save: "save_plugin",
      delete: "delete_plugin",
      toggle: "toggle_plugin",
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

  const getStatus = (id: string): PluginStatus | undefined => {
    return statuses.find((s) => s.id === id);
  };

  const handleRestart = async (id: string) => {
    try {
      await invoke("restart_plugin", { id });
      await loadStatuses();
    } catch (e) {
      console.error("Failed to restart plugin:", e);
    }
  };

  return (
    <div>
      <div className="toolbar">
        <span>{plugins.length} plugin(s)</span>
        <div className="toolbar-actions">
          <button className="btn btn-primary" onClick={handleNew}>
            + New Plugin
          </button>
        </div>
      </div>

      {plugins.length === 0 ? (
        <div className="empty-state">
          <p>No plugins configured</p>
          <button className="btn btn-primary" onClick={handleNew}>
            Register your first plugin
          </button>
        </div>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Enabled</th>
              <th>Name</th>
              <th>Executable</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
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
                  <td>
                    {status?.running ? (
                      <span className="badge badge-success">Running (PID: {status.pid})</span>
                    ) : status?.error ? (
                      <span className="badge badge-error">{status.error}</span>
                    ) : (
                      <span className="badge badge-running">Stopped</span>
                    )}
                  </td>
                  <td>
                    <button
                      className="btn btn-sm btn-secondary"
                      onClick={() => handleRestart(p.id)}
                    >
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
          </tbody>
        </table>
      )}

      {showForm && (
        <PluginForm
          plugin={editingPlugin}
          onSave={(plugin) => handleSave("plugin", plugin)}
          onCancel={closeForm}
        />
      )}
    </div>
  );
}

export default PluginsTab;
