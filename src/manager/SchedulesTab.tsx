import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ScheduleConfig } from "../types";
import ScheduleForm from "./ScheduleForm";

function formatExpression(schedule: ScheduleConfig): string {
  const expr = schedule.expression;
  if (expr.type === "Cron") return `Cron: ${expr.expression}`;
  if (expr.type === "Interval") return `Every ${expr.seconds}s`;
  if (expr.type === "DailyAt") return `Daily at ${expr.time}`;
  return "Unknown";
}

function SchedulesTab() {
  const [schedules, setSchedules] = useState<ScheduleConfig[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingSchedule, setEditingSchedule] = useState<ScheduleConfig | null>(null);

  const loadSchedules = async () => {
    try {
      const data = await invoke<ScheduleConfig[]>("get_schedules");
      setSchedules(data);
    } catch (e) {
      console.error("Failed to load schedules:", e);
    }
  };

  useEffect(() => {
    loadSchedules();
  }, []);

  const handleSave = async (schedule: ScheduleConfig) => {
    try {
      await invoke("save_schedule", { schedule });
      setShowForm(false);
      setEditingSchedule(null);
      await loadSchedules();
    } catch (e) {
      console.error("Failed to save schedule:", e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke("delete_schedule", { id });
      await loadSchedules();
    } catch (e) {
      console.error("Failed to delete schedule:", e);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await invoke("toggle_schedule", { id, enabled });
      await loadSchedules();
    } catch (e) {
      console.error("Failed to toggle schedule:", e);
    }
  };

  const handleTestRun = async (id: string) => {
    try {
      await invoke("test_run_schedule", { id });
    } catch (e) {
      console.error("Failed to test run schedule:", e);
    }
  };

  const handleEdit = (schedule: ScheduleConfig) => {
    setEditingSchedule(schedule);
    setShowForm(true);
  };

  const handleNew = () => {
    setEditingSchedule(null);
    setShowForm(true);
  };

  return (
    <div>
      <div className="toolbar">
        <span>{schedules.length} schedule(s)</span>
        <div className="toolbar-actions">
          <button className="btn btn-primary" onClick={handleNew}>
            + New Schedule
          </button>
        </div>
      </div>

      {schedules.length === 0 ? (
        <div className="empty-state">
          <p>No schedules configured</p>
          <button className="btn btn-primary" onClick={handleNew}>
            Create your first schedule
          </button>
        </div>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Enabled</th>
              <th>Name</th>
              <th>Schedule</th>
              <th>Prompt</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {schedules.map((s) => (
              <tr key={s.id}>
                <td>
                  <button
                    className={`toggle ${s.enabled ? "active" : ""}`}
                    onClick={() => handleToggle(s.id, !s.enabled)}
                  />
                </td>
                <td>{s.name}</td>
                <td>{formatExpression(s)}</td>
                <td
                  style={{
                    maxWidth: 200,
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {s.prompt}
                </td>
                <td>
                  <button className="btn btn-sm btn-secondary" onClick={() => handleTestRun(s.id)}>
                    Run
                  </button>{" "}
                  <button className="btn btn-sm btn-secondary" onClick={() => handleEdit(s)}>
                    Edit
                  </button>{" "}
                  <button className="btn btn-sm btn-danger" onClick={() => handleDelete(s.id)}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {showForm && (
        <ScheduleForm
          schedule={editingSchedule}
          onSave={handleSave}
          onCancel={() => {
            setShowForm(false);
            setEditingSchedule(null);
          }}
        />
      )}
    </div>
  );
}

export default SchedulesTab;
