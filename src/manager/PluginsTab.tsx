import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PluginConfig, PluginStatus } from "../types";
import PluginForm from "./PluginForm";

function PluginsTab() {
  const [plugins, setPlugins] = useState<PluginConfig[]>([]);
  const [statuses, setStatuses] = useState<PluginStatus[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingPlugin, setEditingPlugin] = useState<PluginConfig | null>(null);

  const loadPlugins = async () => {
    try {
      const data = await invoke<PluginConfig[]>("get_plugins");
      setPlugins(data);
    } catch (e) {
      console.error("Failed to load plugins:", e);
    }
  };

  const loadStatuses = async () => {
    try {
      const data = await invoke<PluginStatus[]>("get_plugin_statuses");
      setStatuses(data);
    } catch (e) {
      console.error("Failed to load plugin statuses:", e);
    }
  };

  useEffect(() => {
    loadPlugins();
    loadStatuses();
    const unlisten = listen("plugin-status-changed", () => {
      loadStatuses();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const getStatus = (id: string): PluginStatus | undefined => {
    return statuses.find((s) => s.id === id);
  };

  const handleSave = async (plugin: PluginConfig) => {
    try {
      await invoke("save_plugin", { plugin });
      setShowForm(false);
      setEditingPlugin(null);
      await loadPlugins();
      await loadStatuses();
    } catch (e) {
      console.error("Failed to save plugin:", e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke("delete_plugin", { id });
      await loadPlugins();
      await loadStatuses();
    } catch (e) {
      console.error("Failed to delete plugin:", e);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await invoke("toggle_plugin", { id, enabled });
    } catch (e) {
      console.error("Failed to toggle plugin:", e);
    } finally {
      await loadPlugins();
      await loadStatuses();
    }
  };

  const handleRestart = async (id: string) => {
    try {
      await invoke("restart_plugin", { id });
      await loadStatuses();
    } catch (e) {
      console.error("Failed to restart plugin:", e);
    }
  };

  const handleEdit = (plugin: PluginConfig) => {
    setEditingPlugin(plugin);
    setShowForm(true);
  };

  const handleNew = () => {
    setEditingPlugin(null);
    setShowForm(true);
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
          onSave={handleSave}
          onCancel={() => {
            setShowForm(false);
            setEditingPlugin(null);
          }}
        />
      )}
    </div>
  );
}

export default PluginsTab;
