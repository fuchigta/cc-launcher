import type { ExecutionLog } from "./types";

export function formatSource(log: ExecutionLog): string {
  if (log.source.type === "Schedule") return `Schedule: ${log.source.name}`;
  if (log.source.type === "Plugin") return `Plugin: ${log.source.pluginName}`;
  return "Manual";
}

export function formatDuration(ms: number | null): string {
  if (ms === null) return "-";
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

export function splitArgs(str: string): string[] {
  return str.trim() ? str.trim().split(/\s+/) : [];
}

export function statusBadgeClass(status: string): string {
  if (status === "Success") return "badge badge-success";
  if (status === "Failed") return "badge badge-error";
  return "badge badge-running";
}
