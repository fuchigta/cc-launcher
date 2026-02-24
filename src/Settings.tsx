import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AppConfig, TerminalInfo, TerminalType, WslShell } from "./types";
import { getConfig, saveConfig, getAvailableTerminals } from "./commands";

function Settings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [terminals, setTerminals] = useState<TerminalInfo[]>([]);
  const [shortcutInput, setShortcutInput] = useState("");
  const [selectedTerminal, setSelectedTerminal] = useState<TerminalType>("Auto");
  const [selectedWslShell, setSelectedWslShell] = useState<WslShell>("Bash");
  const [timeoutInput, setTimeoutInput] = useState<number>(3600);
  const [status, setStatus] = useState<string>("");
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    loadConfig();
    loadTerminals();
  }, []);

  const loadConfig = async () => {
    try {
      const cfg = await getConfig();
      setConfig(cfg);
      setShortcutInput(cfg.shortcut);
      setSelectedTerminal(cfg.terminal);
      setSelectedWslShell(cfg.wslShell ?? "Bash");
      setTimeoutInput(cfg.timeoutSecs ?? 3600);
    } catch (error) {
      console.error("Failed to load config:", error);
    }
  };

  const loadTerminals = async () => {
    try {
      const terms = await getAvailableTerminals();
      setTerminals(terms);
    } catch (error) {
      console.error("Failed to load terminals:", error);
    }
  };

  const handleShortcutKeyDown = (e: React.KeyboardEvent) => {
    if (!isRecording) return;

    e.preventDefault();

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");
    if (e.metaKey) parts.push("Win");

    const key = e.key;
    if (key !== "Control" && key !== "Shift" && key !== "Alt" && key !== "Meta") {
      // Normalize key name
      let keyName = key;
      if (key === " ") keyName = "Space";
      else if (key.length === 1) keyName = key.toUpperCase();
      else if (key.startsWith("Arrow")) keyName = key.replace("Arrow", "");

      parts.push(keyName);
      setShortcutInput(parts.join("+"));
      setIsRecording(false);
    }
  };

  const handleSave = async () => {
    if (!config) return;

    const newConfig: AppConfig = {
      ...config,
      shortcut: shortcutInput,
      terminal: selectedTerminal,
      wslShell: selectedWslShell,
      timeoutSecs: timeoutInput,
    };

    try {
      await saveConfig(newConfig);
      setStatus("Settings saved! Restart to apply shortcut changes.");
      setTimeout(() => setStatus(""), 3000);
    } catch (error) {
      setStatus(`Error: ${error}`);
    }
  };

  const handleClose = async () => {
    const window = getCurrentWindow();
    await window.hide();
  };

  const getBestTerminalLabel = (): string => {
    const available = terminals.filter((t) => t.available && t.terminal_type !== "Cmd");
    if (available.length > 0) {
      return `Auto (${available[0].display_name})`;
    }
    return "Auto (Command Prompt)";
  };

  if (!config) {
    return <div className="settings-container">Loading...</div>;
  }

  return (
    <div className="settings-container">
      <h2>Settings</h2>

      <div className="settings-section">
        <label>Global Shortcut</label>
        <div className="shortcut-input-container">
          <input
            type="text"
            value={shortcutInput}
            onKeyDown={handleShortcutKeyDown}
            onFocus={() => setIsRecording(true)}
            onBlur={() => setIsRecording(false)}
            placeholder={isRecording ? "Press keys..." : "Click to record"}
            readOnly={!isRecording}
            className="shortcut-input"
          />
          <button
            type="button"
            onClick={() => setIsRecording(!isRecording)}
            className="record-button"
          >
            {isRecording ? "Recording..." : "Record"}
          </button>
        </div>
        <small>Current: {config.shortcut}</small>
      </div>

      <div className="settings-section">
        <label>Terminal</label>
        <select
          value={selectedTerminal}
          onChange={(e) => setSelectedTerminal(e.target.value as TerminalType)}
          className="terminal-select"
        >
          <option value="Auto">{getBestTerminalLabel()}</option>
          {terminals.map((term) => (
            <option key={term.terminal_type} value={term.terminal_type} disabled={!term.available}>
              {term.display_name} {!term.available && "(not installed)"}
            </option>
          ))}
        </select>
      </div>

      {selectedTerminal === "Wsl" && (
        <div className="settings-section">
          <label>WSL Shell</label>
          <select
            value={selectedWslShell}
            onChange={(e) => setSelectedWslShell(e.target.value as WslShell)}
            className="terminal-select"
          >
            <option value="Bash">Bash</option>
            <option value="Zsh">Zsh</option>
            <option value="Sh">Sh</option>
          </select>
        </div>
      )}

      <div className="settings-section">
        <label>Execution Timeout (seconds)</label>
        <input
          type="number"
          min={60}
          value={timeoutInput}
          onChange={(e) => setTimeoutInput(Number(e.target.value))}
          className="terminal-select"
        />
        <small>Default: 3600 (1 hour)</small>
      </div>

      {status && <div className="status-message">{status}</div>}

      <div className="button-group">
        <button onClick={handleSave} className="save-button">
          Save
        </button>
        <button onClick={handleClose} className="cancel-button">
          Close
        </button>
      </div>
    </div>
  );
}

export default Settings;
