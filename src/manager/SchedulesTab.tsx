import type { ScheduleConfig } from "../types";
import { useCrudTab } from "../hooks/useCrudTab";
import {
  getSchedules,
  saveSchedule,
  deleteSchedule,
  toggleSchedule,
  testRunSchedule,
} from "../commands";
import ScheduleForm from "./ScheduleForm";

function formatExpression(schedule: ScheduleConfig): string {
  const expr = schedule.expression;
  switch (expr.type) {
    case "Cron":
      return `Cron: ${expr.expression}`;
    case "Interval":
      return `Every ${expr.seconds}s`;
    case "DailyAt":
      return `Daily at ${expr.time}`;
  }
}

function SchedulesTab() {
  const {
    items: schedules,
    showForm,
    editingItem: editingSchedule,
    handleSave,
    handleDelete,
    handleToggle,
    handleEdit,
    handleNew,
    closeForm,
  } = useCrudTab<ScheduleConfig>({
    getAll: getSchedules,
    save: saveSchedule,
    delete: deleteSchedule,
    toggle: toggleSchedule,
  });

  const handleTestRun = async (id: string) => {
    try {
      await testRunSchedule(id);
    } catch (e) {
      console.error("Failed to test run schedule:", e);
    }
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
                <td className="truncated-cell">{s.prompt}</td>
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
          onSave={(schedule) => handleSave(schedule)}
          onCancel={closeForm}
        />
      )}
    </div>
  );
}

export default SchedulesTab;
