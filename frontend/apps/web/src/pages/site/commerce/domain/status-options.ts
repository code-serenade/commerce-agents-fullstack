import type { CommerceTranslate } from "../components/commerce-types"

const statusLabelKeys: Record<string, string> = {
  active: "commerce.statusOptions.active",
  archived: "commerce.statusOptions.archived",
  approved: "commerce.statusOptions.approved",
  blocked: "commerce.statusOptions.blocked",
  cancelled: "commerce.statusOptions.cancelled",
  completed: "commerce.statusOptions.completed",
  configured: "commerce.statusOptions.configured",
  critical: "commerce.statusOptions.critical",
  degraded: "commerce.statusOptions.degraded",
  deprecated: "commerce.statusOptions.deprecated",
  dismissed: "commerce.statusOptions.dismissed",
  disabled: "commerce.statusOptions.disabled",
  done: "commerce.statusOptions.done",
  draft: "commerce.statusOptions.draft",
  empty: "commerce.statusOptions.empty",
  enabled: "commerce.statusOptions.enabled",
  ended: "commerce.statusOptions.ended",
  executed: "commerce.statusOptions.executed",
  failed: "commerce.statusOptions.failed",
  idle: "commerce.statusOptions.idle",
  in_progress: "commerce.statusOptions.inProgress",
  missing: "commerce.statusOptions.missing",
  partial: "commerce.statusOptions.partial",
  pending: "commerce.statusOptions.pending",
  pending_approval: "commerce.statusOptions.pendingApproval",
  planned: "commerce.statusOptions.planned",
  paused: "commerce.statusOptions.paused",
  ready: "commerce.statusOptions.ready",
  rejected: "commerce.statusOptions.rejected",
  running: "commerce.statusOptions.running",
  skipped: "commerce.statusOptions.skipped",
  snoozed: "commerce.statusOptions.snoozed",
  success: "commerce.statusOptions.success",
  succeeded: "commerce.statusOptions.succeeded",
  syncing: "commerce.statusOptions.syncing",
  unsupported: "commerce.statusOptions.unsupported",
  warning: "commerce.statusOptions.warning",
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
  return labelKey ? t(labelKey) : t("commerce.enumLabels.other")
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
