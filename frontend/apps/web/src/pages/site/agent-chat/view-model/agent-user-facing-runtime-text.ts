import type { TFunction } from "i18next"
import { normalizeRuntimeRole } from "./agent-process-runtime-view-model"

const DATA_DIAGNOSIS = cjk(25968, 25454, 35786, 26029)
const CREATIVE_OUTPUT = cjk(21019, 24847, 20135, 20986)
const EXPERIENCE_DESIGN = cjk(20307, 39564, 35774, 35745)
const OPS_FINAL = cjk(36816, 33829, 32508, 21512, 26041, 26696)
const AS_ROLE = cjk(20316, 20026)
const ROLE_POST = cjk(23703, 20301)

const INTERNAL_MARKERS = [
  "adapter",
  "external search provider",
  "failure_kind",
  "graph_id",
  "http://",
  "https://",
  "json template validation",
  "json_template_validation",
  "llm_",
  "mark_agent_",
  "missing key",
  "node_id",
  "not allowed for role",
  "not_found",
  "primary_runtime_role",
  "raw=",
  "reasoning_summary",
  "request_id",
  "role_output",
  "role output lacks",
  "role should provide",
  "route_mode",
  "search_evidence",
  "search_trends",
  "selected_skills",
  "skill:",
  "skill_id",
  "structured role output",
  "strategy",
  "task_graph",
  "task_node_",
  "upstream model service",
  "validation_error",
  "worker",
  "worker.",
  "worker_id",
  "worker_unavailable",
  "workspace_run_",
]

export function sanitizeUserFacingProcessText(value: string, t: TFunction) {
  const text = value.trim()
  if (!text) return ""
  if (isInternalRuntimeText(text)) {
    return t("agentChat.thread.runtimeText.genericProgress")
  }
  return text
}

export function formatUserFacingRouteMode(mode: string | undefined, t: TFunction) {
  const normalized = String(mode ?? "").trim() || "single_agent"
  const translated = t(`agentChat.thread.dispatchPanel.routeModes.${normalized}`)
  return translated === `agentChat.thread.dispatchPanel.routeModes.${normalized}` ||
    translated.includes("agentChat.")
    ? t("agentChat.thread.runtimeText.routeFallback")
    : translated
}

export function formatUserFacingSkillStatus(
  status: string | undefined,
  t: TFunction
) {
  const normalized = String(status ?? "").trim().toLowerCase()
  if (["success", "ok", "succeeded", "completed"].includes(normalized)) {
    return t("agentChat.thread.runtimeText.referenceReady")
  }
  if (["failed", "not_found", "validation_error"].includes(normalized)) {
    return t("agentChat.thread.runtimeText.referenceLimited")
  }
  if (["pending", "pending_approval", "requested", "deferred"].includes(normalized)) {
    return t("agentChat.thread.runtimeText.referencePending")
  }
  return t("agentChat.thread.runtimeText.referenceSynced")
}

export function formatUserFacingStepTitle(title: string, t: TFunction) {
  const trimmed = title.trim()
  const normalized = normalizeRuntimeRole(trimmed)
  if (trimmed === DATA_DIAGNOSIS || normalized === "data") {
    return t("agentChat.thread.steps.userFacingTitles.data")
  }
  if (trimmed === CREATIVE_OUTPUT || normalized === "creative") {
    return t("agentChat.thread.steps.userFacingTitles.creative")
  }
  if (trimmed === EXPERIENCE_DESIGN || normalized === "design") {
    return t("agentChat.thread.steps.userFacingTitles.design")
  }
  if (trimmed === OPS_FINAL || normalized === "ops") {
    return t("agentChat.thread.steps.userFacingTitles.ops")
  }
  if (normalized === "search") {
    return t("agentChat.thread.steps.userFacingTitles.search")
  }
  if (isInternalRuntimeText(trimmed)) {
    return t("agentChat.thread.steps.userFacingTitles.generic")
  }
  return trimmed
}

export function formatUserFacingStepDetail(detail: string, t: TFunction) {
  const text = detail.trim()
  if (!text) return t("agentChat.thread.steps.userFacingDetail")
  if (text.length > 180 || isInternalRuntimeText(text)) {
    return t("agentChat.thread.steps.userFacingDetail")
  }
  return text
}

export function isInternalRuntimeText(value: string) {
  const text = value.trim()
  if (!text) return false
  const lower = text.toLowerCase()
  return (
    INTERNAL_MARKERS.some((marker) => lower.includes(marker)) ||
    text.includes(AS_ROLE) ||
    text.includes(ROLE_POST)
  )
}

function cjk(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
