import type { ExecutionLog } from "../types";
import { resumeClaudeSession } from "../commands";
import { formatSource, formatDuration, statusBadgeClass } from "../utils";

interface LogDetailProps {
  log: ExecutionLog;
  onBack: () => void;
}

function LogDetail({ log, onBack }: LogDetailProps) {
  return (
    <div className="log-detail">
      <div className="log-detail-header">
        <button className="btn btn-secondary btn-sm" onClick={onBack}>
          Back
        </button>
        <div className="log-detail-header-right">
          <span className={statusBadgeClass(log.status)}>{log.status}</span>
          {log.sessionId && (
            <button
              className="btn btn-primary btn-sm"
              onClick={() => resumeClaudeSession(log.sessionId!, log.workingDir)}
            >
              Resume
            </button>
          )}
        </div>
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
        <dd>{formatDuration(log.durationMs)}</dd>
        <dt>Started</dt>
        <dd>{new Date(log.startedAt).toLocaleString()}</dd>
        <dt>Completed</dt>
        <dd>{log.completedAt ? new Date(log.completedAt).toLocaleString() : "-"}</dd>
      </dl>

      <h4 className="log-section-title log-section-title-first">stdout</h4>
      <div className="log-output">{log.stdout || "(empty)"}</div>

      <h4 className="log-section-title">stderr</h4>
      <div className="log-output">{log.stderr || "(empty)"}</div>
    </div>
  );
}

export default LogDetail;
