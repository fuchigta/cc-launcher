import { useState, useEffect } from "react";
import type { SubscriptionConfig } from "../types";
import { splitArgs } from "../utils";
import FormModal from "./FormModal";
import DirectoryInput from "./DirectoryInput";

interface SubscriptionFormProps {
  subscription: SubscriptionConfig | null;
  onSave: (subscription: SubscriptionConfig) => void;
  onCancel: () => void;
}

function SubscriptionForm({ subscription, onSave, onCancel }: SubscriptionFormProps) {
  const [name, setName] = useState("");
  const [pluginName, setPluginName] = useState("");
  const [eventType, setEventType] = useState("*");
  const [promptTemplate, setPromptTemplate] = useState("");
  const [workingDir, setWorkingDir] = useState("");
  const [claudeArgs, setClaudeArgs] = useState("");

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
          value={pluginName}
          onChange={(e) => setPluginName(e.target.value)}
          placeholder="my-plugin"
        />
      </div>

      <div className="form-group">
        <label>Event Type (or * for all)</label>
        <input
          className="form-input"
          value={eventType}
          onChange={(e) => setEventType(e.target.value)}
          placeholder="file_created"
        />
      </div>

      <div className="form-group">
        <label>Prompt Template</label>
        <textarea
          className="form-textarea"
          value={promptTemplate}
          onChange={(e) => setPromptTemplate(e.target.value)}
          placeholder={
            "Use {{key}} for event data variables\ne.g. Review the file at {{file_path}}"
          }
        />
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
    </FormModal>
  );
}

export default SubscriptionForm;
