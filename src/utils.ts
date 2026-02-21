import type { ExecutionLog } from "./types";

export function formatSource(log: ExecutionLog): string {
  switch (log.source.type) {
    case "Schedule":
      return `Schedule: ${log.source.name}`;
    case "Plugin":
      return `Plugin: ${log.source.pluginName}`;
    case "Manual":
      return "Manual";
  }
}

export function formatDuration(ms: number | null): string {
  if (ms === null) return "-";
  return ms < 1000 ? `${ms}ms` : `${(ms / 1000).toFixed(1)}s`;
}

export function splitArgs(str: string): string[] {
  return str.trim() ? str.trim().split(/\s+/) : [];
}

export function statusBadgeClass(status: string): string {
  switch (status) {
    case "Success":
      return "badge badge-success";
    case "Failed":
      return "badge badge-error";
    default:
      return "badge badge-running";
  }
}
