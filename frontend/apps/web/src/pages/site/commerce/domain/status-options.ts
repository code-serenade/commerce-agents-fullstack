import type { CommerceTranslate } from "../components/commerce-types"

const statusLabelKeys: Record<string, string> = {
  active: "commerce.statusOptions.active",
  archived: "commerce.statusOptions.archived",
  approved: "commerce.statusOptions.approved",
  blocked: "commerce.statusOptions.blocked",
  cancelled: "commerce.statusOptions.cancelled",
  completed: "commerce.statusOptions.completed",
  configured: "commerce.statusOptions.configured",
  disabled: "commerce.statusOptions.disabled",
  done: "commerce.statusOptions.done",
  enabled: "commerce.statusOptions.enabled",
  executed: "commerce.statusOptions.executed",
  failed: "commerce.statusOptions.failed",
  in_progress: "commerce.statusOptions.inProgress",
  pending: "commerce.statusOptions.pending",
  planned: "commerce.statusOptions.planned",
  ready: "commerce.statusOptions.ready",
  rejected: "commerce.statusOptions.rejected",
  running: "commerce.statusOptions.running",
  skipped: "commerce.statusOptions.skipped",
  succeeded: "commerce.statusOptions.succeeded",
}

export const workspaceTaskStatusValues = [
  "pending",
  "in_progress",
  "done",
  "blocked",
  "cancelled",
] as const

export const workspaceRunStatusValues = [
  "planned",
  "running",
  "succeeded",
  "failed",
  "cancelled",
] as const

export const workspaceRunTaskStatusValues = [
  "pending",
  "running",
  "succeeded",
  "failed",
  "skipped",
  "cancelled",
] as const

export function getCommerceStatusLabel(
  t: CommerceTranslate,
  status: string | null | undefined
) {
  if (!status) return t("commerce.common.none")

  const labelKey = statusLabelKeys[status]
  return labelKey ? t(labelKey) : status.replace(/[_-]+/g, " ")
}

export function commerceStatusOptions(
  t: CommerceTranslate,
  values: readonly string[]
) {
  return values.map((value) => ({
    label: getCommerceStatusLabel(t, value),
    value,
  }))
}

export function workspaceTaskStatusOptions(t: CommerceTranslate) {
  return commerceStatusOptions(t, workspaceTaskStatusValues)
}

export function workspaceRunStatusOptions(t: CommerceTranslate) {
  return commerceStatusOptions(t, workspaceRunStatusValues)
}

export function workspaceRunTaskStatusOptions(t: CommerceTranslate) {
  return commerceStatusOptions(t, workspaceRunTaskStatusValues)
}
