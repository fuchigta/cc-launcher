import { useState, useEffect, useRef } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { PluginConfig, SubscriptionConfig } from "../types";
import { getPlugins } from "../commands";
import { splitArgs } from "../utils";
import FormModal from "./FormModal";
import DirectoryInput from "./DirectoryInput";

interface SubscriptionFormProps {
  subscription: SubscriptionConfig | null;
  onSave: (subscription: SubscriptionConfig) => void;
  onCancel: () => void;
}

// Known event types and their template variables for built-in plugins
const BUILTIN_EVENTS: Record<string, Record<string, string[]>> = {
  "sidecar:folder-watcher": {
    file_created: ["{{file_path}}", "{{timestamp}}"],
    file_changed: ["{{file_path}}", "{{timestamp}}"],
    file_deleted: ["{{file_path}}", "{{timestamp}}"],
    file_renamed: ["{{old_path}}", "{{new_path}}", "{{timestamp}}"],
  },
  "sidecar:imap-watcher": {
    new_mail: [
      "{{message_id}}",
      "{{from}}",
      "{{subject}}",
      "{{date}}",
      "{{body_text}}",
      "{{body_html}}",
      "{{timestamp}}",
    ],
  },
};

function getEventList(executable: string): string[] {
  return Object.keys(BUILTIN_EVENTS[executable] ?? {});
}

function getVarList(executable: string, eventType: string): string[] {
  const events = BUILTIN_EVENTS[executable];
  if (!events) return [];
  return events[eventType] ?? events[Object.keys(events)[0]] ?? [];
}

function SubscriptionForm({ subscription, onSave, onCancel }: SubscriptionFormProps) {
  const [name, setName] = useState("");
  const [pluginName, setPluginName] = useState("");
  const [eventType, setEventType] = useState("*");
  const [promptTemplate, setPromptTemplate] = useState("");
  const [workingDir, setWorkingDir] = useState("");
  const [claudeArgs, setClaudeArgs] = useState("");
  const [plugins, setPlugins] = useState<PluginConfig[]>([]);
  const templateRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    getPlugins()
      .then(setPlugins)
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (subscription) {
      setName(subscription.name);
      setPluginName(subscription.pluginName);
      setEventType(subscription.eventType);
      setPromptTemplate(subscription.promptTemplate);
      setWorkingDir(subscription.workingDir ?? "");
      setClaudeArgs(subscription.claudeArgs.join(" "));
    }
  }, [subscription]);

  const handleSubmit = () => {
    onSave({
      id: subscription?.id ?? crypto.randomUUID(),
      name,
      pluginName,
      eventType,
      promptTemplate,
      workingDir: workingDir || null,
      claudeArgs: splitArgs(claudeArgs),
      enabled: subscription?.enabled ?? true,
    });
  };

  // Detect if selected plugin is a built-in with known event schema
  const selectedPlugin = plugins.find((p) => p.name === pluginName);
  const builtinExecutable = selectedPlugin
    ? BUILTIN_EVENTS[selectedPlugin.executable]
      ? selectedPlugin.executable
      : null
    : null;

  const eventCandidates = builtinExecutable ? getEventList(builtinExecutable) : [];
  const varCandidates =
    builtinExecutable && eventType !== "*"
      ? getVarList(builtinExecutable, eventType)
      : builtinExecutable
        ? getVarList(builtinExecutable, Object.keys(BUILTIN_EVENTS[builtinExecutable])[0])
        : [];

  const insertVar = (v: string) => {
    const el = templateRef.current;
    if (el) {
      const start = el.selectionStart;
      const end = el.selectionEnd;
      const before = promptTemplate.slice(0, start);
      const after = promptTemplate.slice(end);
      const updated = before + v + after;
      setPromptTemplate(updated);
      // Restore cursor after the inserted variable
      requestAnimationFrame(() => {
        el.focus();
        el.setSelectionRange(start + v.length, start + v.length);
      });
    } else {
      setPromptTemplate((prev) => prev + v);
    }
  };

  return (
    <FormModal
      title={subscription ? "Edit Subscription" : "New Subscription"}
      onCancel={onCancel}
      onSave={handleSubmit}
      saveDisabled={!name || !pluginName || !promptTemplate}
    >
      <div className="form-group">
        <label>Name</label>
        <input
          className="form-input"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="My subscription"
        />
      </div>

      <div className="form-group">
        <label>Plugin Name (or * for all)</label>
        <input
          className="form-input"
          list="plugin-name-list"
          value={pluginName}
          onChange={(e) => setPluginName(e.target.value)}
          placeholder="my-plugin"
        />
        <datalist id="plugin-name-list">
          <option value="*" />
          {plugins.map((p) => (
            <option key={p.id} value={p.name} />
          ))}
        </datalist>
      </div>

      <div className="form-group">
        <label>Event Type (or * for all)</label>
        <input
          className="form-input"
          list="event-type-list"
          value={eventType}
          onChange={(e) => setEventType(e.target.value)}
          placeholder="file_created"
        />
        <datalist id="event-type-list">
          <option value="*" />
          {eventCandidates.map((ev) => (
            <option key={ev} value={ev} />
          ))}
        </datalist>
        {eventCandidates.length > 0 && (
          <div className="event-chips" style={{ marginTop: 6 }}>
            {eventCandidates.map((ev) => (
              <span
                key={ev}
                className="event-chip"
                style={{ cursor: "pointer" }}
                onClick={() => setEventType(ev)}
              >
                {ev}
              </span>
            ))}
          </div>
        )}
      </div>

      <div className="form-group">
        <label>Prompt Template</label>
        <textarea
          ref={templateRef}
          className="form-textarea"
          value={promptTemplate}
          onChange={(e) => setPromptTemplate(e.target.value)}
          placeholder={
            "Use {{key}} for event data variables\ne.g. Review the file at {{file_path}}"
          }
        />
        {varCandidates.length > 0 && (
          <div>
            <div style={{ fontSize: 11, color: "#888", marginTop: 6 }}>
              使用可能な変数（クリックでカーソル位置に挿入）:
            </div>
            <div className="var-chips">
              {varCandidates.map((v) => (
                <button key={v} className="var-chip" type="button" onClick={() => insertVar(v)}>
                  {v}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      <DirectoryInput value={workingDir} onChange={setWorkingDir} />

      <div className="form-group">
        <label>Extra Claude Args</label>
        <input
          className="form-input"
          value={claudeArgs}
          onChange={(e) => setClaudeArgs(e.target.value)}
          placeholder="--model sonnet (optional)"
        />
      </div>

      <button
        type="button"
        className="form-help-link"
        onClick={() => openUrl("https://github.com/fuchigta/cc-launcher/blob/main/docs/plugins.md")}
      >
        プラグインイベントのリファレンス →
      </button>
    </FormModal>
  );
}

export default SubscriptionForm;
