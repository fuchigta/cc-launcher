import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { ScheduleConfig, ScheduleExpression } from "../types";

interface ScheduleFormProps {
  schedule: ScheduleConfig | null;
  onSave: (schedule: ScheduleConfig) => void;
  onCancel: () => void;
}

function ScheduleForm({ schedule, onSave, onCancel }: ScheduleFormProps) {
  const [name, setName] = useState("");
  const [exprType, setExprType] = useState<"Cron" | "Interval" | "DailyAt">("Interval");
  const [cronExpr, setCronExpr] = useState("0 0 * * *");
  const [intervalSeconds, setIntervalSeconds] = useState(3600);
  const [dailyTime, setDailyTime] = useState("09:00");
  const [prompt, setPrompt] = useState("");
  const [workingDir, setWorkingDir] = useState("");
  const [claudeArgs, setClaudeArgs] = useState("");

  useEffect(() => {
    if (schedule) {
      setName(schedule.name);
      setPrompt(schedule.prompt);
      setWorkingDir(schedule.workingDir ?? "");
      setClaudeArgs(schedule.claudeArgs.join(" "));
      if (schedule.expression.type === "Cron") {
        setExprType("Cron");
        setCronExpr(schedule.expression.expression);
      } else if (schedule.expression.type === "Interval") {
        setExprType("Interval");
        setIntervalSeconds(schedule.expression.seconds);
      } else if (schedule.expression.type === "DailyAt") {
        setExprType("DailyAt");
        setDailyTime(schedule.expression.time);
      }
    }
  }, [schedule]);

  const handleSubmit = () => {
    let expression: ScheduleExpression;
    if (exprType === "Cron") {
      expression = { type: "Cron", expression: cronExpr };
    } else if (exprType === "Interval") {
      expression = { type: "Interval", seconds: intervalSeconds };
    } else {
      expression = { type: "DailyAt", time: dailyTime };
    }

    const args = claudeArgs.trim() ? claudeArgs.trim().split(/\s+/) : [];

    onSave({
      id: schedule?.id ?? crypto.randomUUID(),
      name,
      expression,
      prompt,
      workingDir: workingDir || null,
      claudeArgs: args,
      enabled: schedule?.enabled ?? true,
    });
  };

  return (
    <div className="form-overlay" onClick={onCancel}>
      <div className="form-dialog" onClick={(e) => e.stopPropagation()}>
        <h3>{schedule ? "Edit Schedule" : "New Schedule"}</h3>

        <div className="form-group">
          <label>Name</label>
          <input
            className="form-input"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My schedule"
          />
        </div>

        <div className="form-group">
          <label>Type</label>
          <select
            className="form-select"
            value={exprType}
            onChange={(e) => setExprType(e.target.value as "Cron" | "Interval" | "DailyAt")}
          >
            <option value="Interval">Interval</option>
            <option value="Cron">Cron</option>
            <option value="DailyAt">Daily At</option>
          </select>
        </div>

        {exprType === "Cron" && (
          <div className="form-group">
            <label>Cron Expression</label>
            <input
              className="form-input"
              value={cronExpr}
              onChange={(e) => setCronExpr(e.target.value)}
              placeholder="0 0 * * *"
            />
          </div>
        )}

        {exprType === "Interval" && (
          <div className="form-group">
            <label>Interval (seconds)</label>
            <input
              className="form-input"
              type="number"
              min={10}
              value={intervalSeconds}
              onChange={(e) => setIntervalSeconds(Number(e.target.value))}
            />
          </div>
        )}

        {exprType === "DailyAt" && (
          <div className="form-group">
            <label>Time (HH:MM)</label>
            <input
              className="form-input"
              type="time"
              value={dailyTime}
              onChange={(e) => setDailyTime(e.target.value)}
            />
          </div>
        )}

        <div className="form-group">
          <label>Prompt</label>
          <textarea
            className="form-textarea"
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Enter prompt for Claude..."
          />
        </div>

        <div className="form-group">
          <label>Working Directory</label>
          <div className="input-with-button">
            <input
              className="form-input"
              value={workingDir}
              onChange={(e) => setWorkingDir(e.target.value)}
              placeholder="C:\path\to\project (optional)"
            />
            <button
              type="button"
              className="btn btn-secondary"
              onClick={async () => {
                const selected = await open({
                  directory: true,
                  multiple: false,
                  defaultPath: workingDir || undefined,
                });
                if (selected && typeof selected === "string") {
                  setWorkingDir(selected);
                }
              }}
            >
              Browse
            </button>
          </div>
        </div>

        <div className="form-group">
          <label>Extra Claude Args</label>
          <input
            className="form-input"
            value={claudeArgs}
            onChange={(e) => setClaudeArgs(e.target.value)}
            placeholder="--model sonnet (optional)"
          />
        </div>

        <div className="form-actions">
          <button className="btn btn-secondary" onClick={onCancel}>
            Cancel
          </button>
          <button className="btn btn-primary" onClick={handleSubmit} disabled={!name || !prompt}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}

export default ScheduleForm;
