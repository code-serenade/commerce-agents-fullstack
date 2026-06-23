import type { TFunction } from "i18next"

const MODEL_UNAVAILABLE_MARKERS = [
  "llm_provider_error",
  "provider error",
  "upstream model service unavailable",
  "internal server error",
  "api error: status=500",
]

const MODEL_CONFIG_MARKERS = [
  "llm access denied",
  "llm_access_denied",
  "unauthorized",
  "forbidden",
]

const WORKER_UNAVAILABLE_MARKERS = [
  "worker_unavailable",
  "worker unavailable",
  "no worker",
]

const COLLABORATION_INCOMPLETE_MARKERS = [
  "adapter",
  "accounting role output lacks",
  "accounting role should provide",
  "creative role output lacks",
  "creative role should provide",
  "data role should provide",
  "design role output lacks",
  "design role should provide",
  "engineering role output lacks",
  "engineering role should provide",
  "ops role should provide",
  "service role output lacks",
  "service role should provide",
  "worker reported failed status",
  "web role output lacks",
  "web role should provide",
  "structured role output is missing",
  "content must be specific and non-empty",
  "json template validation",
  "decode failed",
  "cognition failure",
  "external search provider",
  "raw=",
  "missing key",
  "failure_kind",
  "graph_id",
  "role_output",
  "route_mode",
  "search_evidence",
  "search_trends",
  "selected_skills",
  "skill '",
  'skill "',
  "skill_id",
  "not allowed for role",
  "strategy",
  "task_graph",
  "worker.",
  "worker_id",
]

const TIMEOUT_MARKERS = [
  "timed out",
  "timeout",
]

export function formatUserFacingRuntimeReason(
  reason: string | undefined | null,
  t: TFunction
) {
  const text = String(reason ?? "").trim()
  if (!text) return t("agentChat.swarm.runtimeReasons.collaborationIncomplete")

  const lower = text.toLowerCase()
  if (containsAny(lower, MODEL_UNAVAILABLE_MARKERS)) {
    return t("agentChat.swarm.runtimeReasons.modelUnavailable")
  }
  if (containsAny(lower, MODEL_CONFIG_MARKERS)) {
    return t("agentChat.swarm.runtimeReasons.modelConfigUnavailable")
  }
  if (containsAny(lower, WORKER_UNAVAILABLE_MARKERS)) {
    return t("agentChat.swarm.runtimeReasons.workerUnavailable")
  }
  if (containsAny(lower, TIMEOUT_MARKERS)) {
    return t("agentChat.swarm.runtimeReasons.timedOut")
  }
  if (isInternalRuntimeReason(text)) {
    return t("agentChat.swarm.runtimeReasons.collaborationIncomplete")
  }

  return text
}

export function isInternalRuntimeReason(value: string) {
  const lower = value.trim().toLowerCase()
  if (!lower) return false
  if (lower.includes("http://") || lower.includes("https://")) return true
  return containsAny(lower, [
    ...MODEL_UNAVAILABLE_MARKERS,
    ...MODEL_CONFIG_MARKERS,
    ...WORKER_UNAVAILABLE_MARKERS,
    ...COLLABORATION_INCOMPLETE_MARKERS,
  ])
}

function containsAny(value: string, markers: string[]) {
  return markers.some((marker) => value.includes(marker))
}
