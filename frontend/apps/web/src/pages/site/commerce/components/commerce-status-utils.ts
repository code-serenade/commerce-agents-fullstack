export type StatusTone = "default" | "success" | "warning" | "danger" | "muted"

export function toneForStatus(status?: string | null): StatusTone {
  if (!status) return "muted"
  if (
    [
      "active",
      "enabled",
      "configured",
      "success",
      "succeeded",
      "completed",
      "done",
      "approved",
      "executed",
      "acted",
      "ready",
    ].includes(status)
  ) {
    return "success"
  }
  if (
    [
      "pending",
      "draft",
      "planned",
      "running",
      "syncing",
      "in_progress",
      "snoozed",
    ].includes(status)
  ) {
    return "warning"
  }
  if (
    [
      "failed",
      "cancelled",
      "rejected",
      "critical",
      "dismissed",
      "archived",
      "disabled",
      "deprecated",
      "blocked",
      "adapter_not_implemented",
      "connection_disabled",
      "connection_missing",
      "missing",
      "unsupported",
      "unsupported_action",
      "validation_error",
    ].includes(status)
  ) {
    return "danger"
  }

  return "default"
}
