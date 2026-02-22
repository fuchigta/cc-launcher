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
import CrudTabLayout from "./CrudTabLayout";

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
    error,
    handleSave,
    handleDelete,
    handleToggle,
    handleEdit,
    handleNew,
    closeForm,
    clearError,
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
    <>
      <CrudTabLayout
        error={error}
        clearError={clearError}
        itemCount={schedules.length}
        itemLabel="schedule"
        newButtonLabel="+ New Schedule"
        emptyMessage="No schedules configured"
        emptyButtonLabel="Create your first schedule"
        onNew={handleNew}
        headers={["Enabled", "Name", "Schedule", "Prompt", "Actions"]}
      >
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
      </CrudTabLayout>

      {showForm && (
        <ScheduleForm
          schedule={editingSchedule}
          onSave={(schedule) => handleSave(schedule)}
          onCancel={closeForm}
        />
      )}
    </>
  );
}

export default SchedulesTab;
