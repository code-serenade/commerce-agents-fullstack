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

const EXTERNAL_INTERFACE_NOT_CONNECTED = cjk(
  22806,
  37096,
  25509,
  21475,
  26410,
  25509,
  20837
)
const EXTERNAL_DATA_NOT_CONNECTED = cjk(
  22806,
  37096,
  25968,
  25454,
  26410,
  25509,
  20837
)
const EXTERNAL_DATA_UNAVAILABLE = cjk(
  22806,
  37096,
  25968,
  25454,
  26242,
  19981,
  21487,
  29992
)
const COLLABORATION_STEP_TIMED_OUT = cjk(
  21327,
  20316,
  27493,
  39588,
  25191,
  34892,
  36229,
  26102
)
const STEP_TIMED_OUT = cjk(27493, 39588, 25191, 34892, 36229, 26102)
const PROCESS_TIMED_OUT = cjk(22788, 29702, 36229, 26102)

const EXTERNAL_DATA_UNAVAILABLE_MARKERS = [
  "external data unavailable",
  "external data not connected",
  "external api unavailable",
  "external service unavailable",
  EXTERNAL_INTERFACE_NOT_CONNECTED,
  EXTERNAL_DATA_NOT_CONNECTED,
  EXTERNAL_DATA_UNAVAILABLE,
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
  COLLABORATION_STEP_TIMED_OUT,
  STEP_TIMED_OUT,
  PROCESS_TIMED_OUT,
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
  if (containsAny(lower, EXTERNAL_DATA_UNAVAILABLE_MARKERS)) {
    return t("agentChat.swarm.runtimeReasons.externalDataUnavailable")
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
    ...EXTERNAL_DATA_UNAVAILABLE_MARKERS,
    ...COLLABORATION_INCOMPLETE_MARKERS,
    ...TIMEOUT_MARKERS,
  ])
}

function containsAny(value: string, markers: string[]) {
  return markers.some((marker) => value.includes(marker))
}

function cjk(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
