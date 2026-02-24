import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ExecutionLog } from "../types";
import { formatSource, formatDuration, statusBadgeClass } from "../utils";
import { getLogs, clearLogs } from "../commands";
import LogDetail from "./LogDetail";

function LogsTab() {
  const [logs, setLogs] = useState<ExecutionLog[]>([]);
  const [selectedLog, setSelectedLog] = useState<ExecutionLog | null>(null);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  const loadLogs = async () => {
    try {
      const data = await getLogs(limit, offset);
      setLogs(data);
    } catch (e) {
      console.error("Failed to load logs:", e);
    }
  };

  useEffect(() => {
    loadLogs();
    const unlistenStarted = listen("execution-started", () => {
      loadLogs();
    });
    const unlistenCompleted = listen("execution-completed", () => {
      loadLogs();
    });
    return () => {
      unlistenStarted.then((fn) => fn());
      unlistenCompleted.then((fn) => fn());
    };
  }, [offset]);

  const handleClear = async () => {
    try {
      await clearLogs();
      setLogs([]);
      setSelectedLog(null);
    } catch (e) {
      console.error("Failed to clear logs:", e);
    }
  };

  if (selectedLog) {
    return <LogDetail log={selectedLog} onBack={() => setSelectedLog(null)} />;
  }

  return (
    <div>
      <div className="toolbar">
        <span>{logs.length} log(s)</span>
        <div className="toolbar-actions">
          <button className="btn btn-sm btn-danger" onClick={handleClear}>
            Clear All
          </button>
        </div>
      </div>

      {logs.length === 0 ? (
        <div className="empty-state">
          <p>No execution logs yet</p>
        </div>
      ) : (
        <>
          <table className="data-table">
            <thead>
              <tr>
                <th>Status</th>
                <th>Source</th>
                <th>Prompt</th>
                <th>Duration</th>
                <th>Started</th>
              </tr>
            </thead>
            <tbody>
              {logs.map((log) => (
                <tr key={log.id} style={{ cursor: "pointer" }} onClick={() => setSelectedLog(log)}>
                  <td>
                    <span className={statusBadgeClass(log.status)}>{log.status}</span>
                  </td>
                  <td>{formatSource(log)}</td>
                  <td className="truncated-cell-wide">{log.prompt}</td>
                  <td>{formatDuration(log.durationMs)}</td>
                  <td>{new Date(log.startedAt).toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>

          <div className="pagination">
            <button
              className="btn btn-sm btn-secondary"
              disabled={offset === 0}
              onClick={() => setOffset(Math.max(0, offset - limit))}
            >
              Previous
            </button>
            <button
              className="btn btn-sm btn-secondary"
              disabled={logs.length < limit}
              onClick={() => setOffset(offset + limit)}
            >
              Next
            </button>
          </div>
        </>
      )}
    </div>
  );
}

export default LogsTab;
