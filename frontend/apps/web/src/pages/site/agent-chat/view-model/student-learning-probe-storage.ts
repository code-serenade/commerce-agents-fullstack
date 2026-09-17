import type { LearningProbeResponse } from "@/api"

export interface StoredEngagedProbe {
  id: string
  questionText: string
  questionType: "single_choice" | "open_response"
  choices: Array<{ id: string; text: string }>
  interactionMode: "contextual_micro"
  rewardCode: "progress_feedback"
  engaged: true
}

export function toStoredProbe(
  probe: LearningProbeResponse
): StoredEngagedProbe {
  return {
    engaged: true,
    id: probe.id,
    interactionMode: probe.interactionMode,
    questionText: probe.questionText,
    questionType:
      probe.questionType === "single_choice"
        ? "single_choice"
        : "open_response",
    choices: Array.isArray(probe.choices) ? probe.choices : [],
    rewardCode: probe.rewardCode,
  }
}

export function readStoredProbe(
  conversationId?: string,
  messageId?: string
): StoredEngagedProbe | null {
  const key = storedProbeKey(conversationId, messageId)
  if (!key || typeof window === "undefined") return null
  try {
    const legacyKey = legacyStoredProbeKey(conversationId)
    const storedValue = window.sessionStorage.getItem(key)
    const legacyValue = legacyKey
      ? window.sessionStorage.getItem(legacyKey)
      : null
    const value = JSON.parse(
      storedValue ?? legacyValue ?? "null"
    ) as Partial<StoredEngagedProbe> | null
    if (
      !value ||
      typeof value.id !== "string" ||
      typeof value.questionText !== "string"
    ) {
      return null
    }
    const restored = {
      engaged: true,
      id: value.id,
      interactionMode: "contextual_micro",
      questionText: value.questionText,
      questionType:
        value.questionType === "single_choice"
          ? "single_choice"
          : "open_response",
      choices: Array.isArray(value.choices)
        ? value.choices.filter(
            (choice): choice is { id: string; text: string } =>
              typeof choice?.id === "string" && typeof choice?.text === "string"
          )
        : [],
      rewardCode: "progress_feedback",
    } satisfies StoredEngagedProbe
    if (!storedValue && legacyValue && legacyKey) {
      window.sessionStorage.setItem(key, JSON.stringify(restored))
      window.sessionStorage.removeItem(legacyKey)
    }
    return restored
  } catch {
    return null
  }
}

export function writeStoredProbe(
  conversationId: string | undefined,
  messageId: string | undefined,
  probe: StoredEngagedProbe
) {
  const key = storedProbeKey(conversationId, messageId)
  if (!key || typeof window === "undefined") return
  try {
    window.sessionStorage.setItem(key, JSON.stringify(probe))
  } catch {
    // Storage can be unavailable in privacy modes; the in-memory flow still works.
  }
}

export function clearStoredProbe(conversationId?: string, messageId?: string) {
  const key = storedProbeKey(conversationId, messageId)
  if (!key || typeof window === "undefined") return
  try {
    window.sessionStorage.removeItem(key)
    const legacyKey = legacyStoredProbeKey(conversationId)
    if (legacyKey) window.sessionStorage.removeItem(legacyKey)
  } catch {
    // Closing or skipping a probe must not depend on browser storage access.
  }
}

function legacyStoredProbeKey(conversationId?: string) {
  return conversationId ? `learning-engaged-probe:${conversationId}` : null
}

function storedProbeKey(conversationId?: string, messageId?: string) {
  return conversationId && messageId
    ? `learning-engaged-probe:${conversationId}:${messageId}`
    : null
}
