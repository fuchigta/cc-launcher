import { useState, useEffect } from "react";
import type { PluginConfig } from "../types";
import { splitArgs } from "../utils";
import FormModal from "./FormModal";

type PluginPreset = "custom" | "folder-watcher" | "imap-watcher";

interface PluginFormProps {
  plugin: PluginConfig | null;
  onSave: (plugin: PluginConfig) => void;
  onCancel: () => void;
}

function detectPreset(plugin: PluginConfig | null): PluginPreset {
  if (!plugin) return "custom";
  if (plugin.executable === "sidecar:folder-watcher") return "folder-watcher";
  if (plugin.executable === "sidecar:imap-watcher") return "imap-watcher";
  return "custom";
}

function parseArgs(args: string[], prefix: string): string | undefined {
  const idx = args.indexOf(prefix);
  if (idx !== -1 && idx + 1 < args.length) return args[idx + 1];
  return undefined;
}

function hasFlag(args: string[], flag: string): boolean {
  return args.includes(flag);
}

// --- Folder Watcher sub-form ---

interface FolderWatcherState {
  dir: string;
  recursive: boolean;
  filter: string;
  ignore: string;
  debounce: string;
}

function parseFolderWatcherArgs(args: string[]): FolderWatcherState {
  return {
    dir: parseArgs(args, "--dir") ?? "",
    recursive: hasFlag(args, "--recursive"),
    filter: parseArgs(args, "--filter") ?? "",
    ignore: parseArgs(args, "--ignore") ?? ".git,node_modules",
    debounce: parseArgs(args, "--debounce") ?? "300",
  };
}

function buildFolderWatcherArgs(state: FolderWatcherState): string[] {
  const args: string[] = ["--dir", state.dir];
  if (state.recursive) args.push("--recursive");
  if (state.filter.trim()) args.push("--filter", state.filter.trim());
  if (state.ignore.trim()) args.push("--ignore", state.ignore.trim());
  if (state.debounce.trim() && state.debounce !== "300") {
    args.push("--debounce", state.debounce.trim());
  }
  return args;
}

function FolderWatcherForm({
  state,
  onChange,
}: {
  state: FolderWatcherState;
  onChange: (s: FolderWatcherState) => void;
}) {
  return (
    <>
      <div className="form-group">
        <label>Directory</label>
        <input
          className="form-input"
          value={state.dir}
          onChange={(e) => onChange({ ...state, dir: e.target.value })}
          placeholder="C:\path\to\watch"
        />
      </div>

      <div className="form-group">
        <label>
          <input
            type="checkbox"
            checked={state.recursive}
            onChange={(e) => onChange({ ...state, recursive: e.target.checked })}
            style={{ marginRight: 8 }}
          />
          Recursive (watch subdirectories)
        </label>
      </div>

      <div className="form-group">
        <label>Filter patterns (comma-separated globs)</label>
        <input
          className="form-input"
          value={state.filter}
          onChange={(e) => onChange({ ...state, filter: e.target.value })}
          placeholder="*.txt,*.csv (empty = all files)"
        />
      </div>

      <div className="form-group">
        <label>Ignore patterns (comma-separated)</label>
        <input
          className="form-input"
          value={state.ignore}
          onChange={(e) => onChange({ ...state, ignore: e.target.value })}
          placeholder=".git,node_modules"
        />
      </div>

      <div className="form-group">
        <label>Debounce (ms)</label>
        <input
          className="form-input"
          type="number"
          value={state.debounce}
          onChange={(e) => onChange({ ...state, debounce: e.target.value })}
          placeholder="300"
        />
      </div>
    </>
  );
}

// --- IMAP Watcher sub-form ---

interface ImapWatcherState {
  server: string;
  port: string;
  user: string;
  password: string;
  folder: string;
  pollInterval: string;
  tls: boolean;
  subjectMatch: string;
  bodyMatch: string;
}

function parseImapWatcherArgs(args: string[]): ImapWatcherState {
  return {
    server: parseArgs(args, "--server") ?? "",
    port: parseArgs(args, "--port") ?? "993",
    user: parseArgs(args, "--user") ?? "",
    password: parseArgs(args, "--password") ?? "",
    folder: parseArgs(args, "--folder") ?? "INBOX",
    pollInterval: parseArgs(args, "--poll-interval") ?? "60",
    tls: !hasFlag(args, "--no-tls"),
    subjectMatch: parseArgs(args, "--subject-match") ?? "",
    bodyMatch: parseArgs(args, "--body-match") ?? "",
  };
}

function buildImapWatcherArgs(state: ImapWatcherState): string[] {
  const args: string[] = [
    "--server",
    state.server,
    "--port",
    state.port,
    "--user",
    state.user,
    "--password",
    state.password,
  ];
  if (state.folder.trim() && state.folder !== "INBOX") {
    args.push("--folder", state.folder.trim());
  }
  if (state.pollInterval.trim() && state.pollInterval !== "60") {
    args.push("--poll-interval", state.pollInterval.trim());
  }
  if (!state.tls) args.push("--no-tls");
  if (state.subjectMatch.trim()) {
    args.push("--subject-match", state.subjectMatch.trim());
  }
  if (state.bodyMatch.trim()) {
    args.push("--body-match", state.bodyMatch.trim());
  }
  return args;
}

function ImapWatcherForm({
  state,
  onChange,
}: {
  state: ImapWatcherState;
  onChange: (s: ImapWatcherState) => void;
}) {
  return (
    <>
      <div className="form-group">
        <label>Server</label>
        <input
          className="form-input"
          value={state.server}
          onChange={(e) => onChange({ ...state, server: e.target.value })}
          placeholder="imap.example.com"
        />
      </div>

      <div className="form-group">
        <label>Port</label>
        <input
          className="form-input"
          type="number"
          value={state.port}
          onChange={(e) => onChange({ ...state, port: e.target.value })}
          placeholder="993"
        />
      </div>

      <div className="form-group">
        <label>User</label>
        <input
          className="form-input"
          value={state.user}
          onChange={(e) => onChange({ ...state, user: e.target.value })}
          placeholder="user@example.com"
        />
      </div>

      <div className="form-group">
        <label>Password</label>
        <input
          className="form-input"
          type="password"
          value={state.password}
          onChange={(e) => onChange({ ...state, password: e.target.value })}
          placeholder="Password"
        />
      </div>

      <div className="form-group">
        <label>Folder</label>
        <input
          className="form-input"
          value={state.folder}
          onChange={(e) => onChange({ ...state, folder: e.target.value })}
          placeholder="INBOX"
        />
      </div>

      <div className="form-group">
        <label>Poll interval (seconds)</label>
        <input
          className="form-input"
          type="number"
          value={state.pollInterval}
          onChange={(e) => onChange({ ...state, pollInterval: e.target.value })}
          placeholder="60"
        />
      </div>

      <div className="form-group">
        <label>
          <input
            type="checkbox"
            checked={state.tls}
            onChange={(e) => onChange({ ...state, tls: e.target.checked })}
            style={{ marginRight: 8 }}
          />
          Use TLS
        </label>
      </div>

      <div className="form-group">
        <label>Subject match (regex, optional)</label>
        <input
          className="form-input"
          value={state.subjectMatch}
          onChange={(e) => onChange({ ...state, subjectMatch: e.target.value })}
          placeholder="e.g. alert|notification"
        />
      </div>

      <div className="form-group">
        <label>Body match (regex, optional)</label>
        <input
          className="form-input"
          value={state.bodyMatch}
          onChange={(e) => onChange({ ...state, bodyMatch: e.target.value })}
          placeholder="e.g. urgent|critical"
        />
      </div>
    </>
  );
}

// --- Main form ---

function PluginForm({ plugin, onSave, onCancel }: PluginFormProps) {
  const [preset, setPreset] = useState<PluginPreset>("custom");
  const [name, setName] = useState("");
  const [executable, setExecutable] = useState("");
  const [args, setArgs] = useState("");

  const [folderState, setFolderState] = useState<FolderWatcherState>({
    dir: "",
    recursive: false,
    filter: "",
    ignore: ".git,node_modules",
    debounce: "300",
  });

  const [imapState, setImapState] = useState<ImapWatcherState>({
    server: "",
    port: "993",
    user: "",
    password: "",
    folder: "INBOX",
    pollInterval: "60",
    tls: true,
    subjectMatch: "",
    bodyMatch: "",
  });

  useEffect(() => {
    if (plugin) {
      const detected = detectPreset(plugin);
      setPreset(detected);
      setName(plugin.name);
      setExecutable(plugin.executable);
      setArgs(plugin.args.join(" "));
      if (detected === "folder-watcher") {
        setFolderState(parseFolderWatcherArgs(plugin.args));
      } else if (detected === "imap-watcher") {
        setImapState(parseImapWatcherArgs(plugin.args));
      }
    }
  }, [plugin]);

  const handlePresetChange = (newPreset: PluginPreset) => {
    setPreset(newPreset);
    if (newPreset === "folder-watcher") {
      setExecutable("sidecar:folder-watcher");
      if (!name || name === "ImapWatcher") setName("FolderWatcher");
    } else if (newPreset === "imap-watcher") {
      setExecutable("sidecar:imap-watcher");
      if (!name || name === "FolderWatcher") setName("ImapWatcher");
    } else {
      if (executable === "sidecar:folder-watcher" || executable === "sidecar:imap-watcher") {
        setExecutable("");
      }
    }
  };

  const handleSubmit = () => {
    let finalArgs: string[];
    if (preset === "folder-watcher") {
      finalArgs = buildFolderWatcherArgs(folderState);
    } else if (preset === "imap-watcher") {
      finalArgs = buildImapWatcherArgs(imapState);
    } else {
      finalArgs = splitArgs(args);
    }

    onSave({
      id: plugin?.id ?? crypto.randomUUID(),
      name,
      executable,
      args: finalArgs,
      enabled: plugin?.enabled ?? true,
    });
  };

  const isValid = () => {
    if (!name) return false;
    if (preset === "folder-watcher") return !!folderState.dir;
    if (preset === "imap-watcher")
      return !!imapState.server && !!imapState.user && !!imapState.password;
    return !!executable;
  };

  return (
    <FormModal
      title={plugin ? "Edit Plugin" : "New Plugin"}
      onCancel={onCancel}
      onSave={handleSubmit}
      saveDisabled={!isValid()}
    >
      <div className="form-group">
        <label>Type</label>
        <select
          className="form-select"
          value={preset}
          onChange={(e) => handlePresetChange(e.target.value as PluginPreset)}
        >
          <option value="custom">Custom</option>
          <option value="folder-watcher">Folder Watcher (built-in)</option>
          <option value="imap-watcher">IMAP Watcher (built-in)</option>
        </select>
      </div>

      <div className="form-group">
        <label>Name</label>
        <input
          className="form-input"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="My plugin"
        />
      </div>

      {preset === "custom" && (
        <>
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
        </>
      )}

      {preset === "folder-watcher" && (
        <FolderWatcherForm state={folderState} onChange={setFolderState} />
      )}

      {preset === "imap-watcher" && <ImapWatcherForm state={imapState} onChange={setImapState} />}
    </FormModal>
  );
}

export default PluginForm;
