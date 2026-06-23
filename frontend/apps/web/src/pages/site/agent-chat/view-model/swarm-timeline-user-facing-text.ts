import type { TFunction } from "i18next"
import {
  isInternalRuntimeText,
  sanitizeUserFacingProcessText,
} from "./agent-user-facing-runtime-text"
import { formatUserFacingRuntimeReason } from "./swarm-runtime-reason-view-model"

export function formatTimelineDynamicText(
  value: string | undefined | null,
  t: TFunction
) {
  const text = String(value ?? "").trim()
  if (!text) return t("agentChat.thread.runtimeText.genericProgress")
  if (isInternalRuntimeText(text) || isMachineReadableRuntimeToken(text)) {
    return t("agentChat.thread.runtimeText.genericProgress")
  }
  return sanitizeUserFacingProcessText(
    formatUserFacingRuntimeReason(text, t),
    t
  )
}

export function formatUserFacingTaskTitle(
  value: string | undefined | null,
  t: TFunction
) {
  const text = String(value ?? "").trim()
  if (!text) return t("agentChat.thread.steps.userFacingTitles.generic")
  if (isInternalRuntimeText(text) || isMachineReadableRuntimeToken(text)) {
    return t("agentChat.thread.steps.userFacingTitles.generic")
  }
  return formatTimelineDynamicText(text, t)
}

export function formatUserFacingTaskStatus(
  status: string | undefined,
  t: TFunction
) {
  const normalized = String(status ?? "").trim().toLowerCase()
  if (["completed", "done", "success", "succeeded"].includes(normalized)) {
    return t("agentChat.thread.inlineState.done")
  }
  if (["running", "started", "in_progress", "processing"].includes(normalized)) {
    return t("agentChat.thread.inlineState.working")
  }
  if (["queued", "pending"].includes(normalized)) {
    return t("agentChat.thread.inlineState.queued")
  }
  if (["failed", "error", "blocked"].includes(normalized)) {
    return t("agentChat.thread.inlineState.failed")
  }
  if (isInternalRuntimeText(normalized)) {
    return t("agentChat.thread.runtimeText.genericProgress")
  }
  return formatTimelineDynamicText(status, t)
}

function isMachineReadableRuntimeToken(value: string) {
  return /^[a-z][a-z0-9]+(?:[_-][a-z0-9]+)+$/.test(value.trim())
}
