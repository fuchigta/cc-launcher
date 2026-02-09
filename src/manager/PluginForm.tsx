import { useState, useEffect } from "react";
import type { PluginConfig } from "../types";

interface PluginFormProps {
  plugin: PluginConfig | null;
  onSave: (plugin: PluginConfig) => void;
  onCancel: () => void;
}

function PluginForm({ plugin, onSave, onCancel }: PluginFormProps) {
  const [name, setName] = useState("");
  const [executable, setExecutable] = useState("");
  const [args, setArgs] = useState("");

  useEffect(() => {
    if (plugin) {
      setName(plugin.name);
      setExecutable(plugin.executable);
      setArgs(plugin.args.join(" "));
    }
  }, [plugin]);

  const handleSubmit = () => {
    const argList = args.trim() ? args.trim().split(/\s+/) : [];
    onSave({
      id: plugin?.id ?? crypto.randomUUID(),
      name,
      executable,
      args: argList,
      enabled: plugin?.enabled ?? true,
    });
  };

  return (
    <div className="form-overlay" onClick={onCancel}>
      <div className="form-dialog" onClick={(e) => e.stopPropagation()}>
        <h3>{plugin ? "Edit Plugin" : "New Plugin"}</h3>

        <div className="form-group">
          <label>Name</label>
          <input
            className="form-input"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My plugin"
          />
        </div>

        <div className="form-group">
          <label>Executable</label>
          <input
            className="form-input"
            value={executable}
            onChange={(e) => setExecutable(e.target.value)}
            placeholder="C:\path\to\plugin.exe or node"
          />
        </div>

        <div className="form-group">
          <label>Arguments</label>
          <input
            className="form-input"
            value={args}
            onChange={(e) => setArgs(e.target.value)}
            placeholder="script.js --watch (optional)"
          />
        </div>

        <div className="form-actions">
          <button className="btn btn-secondary" onClick={onCancel}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSubmit}
            disabled={!name || !executable}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}

export default PluginForm;
