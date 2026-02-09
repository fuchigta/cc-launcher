import type { ExecutionLog } from "../types";

interface LogDetailProps {
  log: ExecutionLog;
  onBack: () => void;
}

function formatSource(log: ExecutionLog): string {
  if (log.source.type === "Schedule") return `Schedule: ${log.source.name}`;
  if (log.source.type === "Plugin") return `Plugin: ${log.source.pluginName}`;
  return "Manual";
}

function LogDetail({ log, onBack }: LogDetailProps) {
  return (
    <div className="log-detail">
      <div className="log-detail-header">
        <button className="btn btn-secondary btn-sm" onClick={onBack}>
          Back
        </button>
        <span
          className={`badge ${
            log.status === "Success"
              ? "badge-success"
              : log.status === "Failed"
                ? "badge-error"
                : "badge-running"
          }`}
        >
          {log.status}
        </span>
      </div>

      <dl className="log-meta">
        <dt>Source</dt>
        <dd>{formatSource(log)}</dd>
        <dt>Prompt</dt>
        <dd>{log.prompt}</dd>
        <dt>Working Dir</dt>
        <dd>{log.workingDir ?? "-"}</dd>
        <dt>Exit Code</dt>
        <dd>{log.exitCode ?? "-"}</dd>
        <dt>Duration</dt>
        <dd>{log.durationMs != null ? `${(log.durationMs / 1000).toFixed(1)}s` : "-"}</dd>
        <dt>Started</dt>
        <dd>{new Date(log.startedAt).toLocaleString()}</dd>
        <dt>Completed</dt>
        <dd>{log.completedAt ? new Date(log.completedAt).toLocaleString() : "-"}</dd>
      </dl>

      <h4 style={{ marginTop: 20, marginBottom: 8 }}>stdout</h4>
      <div className="log-output">{log.stdout || "(empty)"}</div>

      <h4 style={{ marginBottom: 8 }}>stderr</h4>
      <div className="log-output">{log.stderr || "(empty)"}</div>
    </div>
  );
}

export default LogDetail;
