import type { TFunction } from "i18next"
import type { AgentDispatchFact } from "./agent-process-kind-view-model"
import { sanitizeUserFacingProcessText } from "./agent-user-facing-runtime-text"

const MAX_DISPATCH_FACTS = 6

export function selectDispatchFacts(
  facts: AgentDispatchFact[],
  t: TFunction
) {
  if (facts.length <= MAX_DISPATCH_FACTS) return sanitizeDispatchFacts(facts, t)

  const selected = new Set<number>()
  addIndex(selected, 0)

  addFirstLabel(selected, facts, t("agentChat.thread.dispatchFacts.roleAudit.label"))
  addFirstLabel(selected, facts, t("agentChat.thread.dispatchFacts.searchAudit.label"))
  addFirstLabel(selected, facts, t("agentChat.thread.dispatchFacts.skillAudit.label"))
  addFirstLabel(selected, facts, t("agentChat.thread.dispatchFacts.materialAudit.label"))
  addFirstLabel(selected, facts, t("agentChat.thread.dispatchFacts.workspaceWarning.label"))
  addFirstLabel(selected, facts, t("agentChat.thread.dispatchFacts.workspaceOmitted.label"))

  facts.forEach((fact, index) => {
    if (fact.tone === "warning" || fact.tone === "error") {
      addIndex(selected, index)
    }
  })

  facts.forEach((_, index) => addIndex(selected, index))

  return Array.from(selected)
    .sort((left, right) => left - right)
    .map((index) => facts[index])
    .filter((fact): fact is AgentDispatchFact => Boolean(fact))
    .map((fact) => sanitizeDispatchFact(fact, t))
}

function sanitizeDispatchFacts(facts: AgentDispatchFact[], t: TFunction) {
  return facts.map((fact) => sanitizeDispatchFact(fact, t))
}

function sanitizeDispatchFact(fact: AgentDispatchFact, t: TFunction) {
  const fallback = getKnownFactFallback(fact, t)
  const label = sanitizeUserFacingProcessText(fact.label, t)
  const detail = sanitizeUserFacingProcessText(fact.detail, t)

  return {
    ...fact,
    detail: fallback.detail ?? detail,
    label: fallback.label ?? (
      label === t("agentChat.thread.runtimeText.genericProgress")
        ? t("agentChat.thread.dispatchFacts.userFacingFallback.label")
        : label
    ),
  }
}

function getKnownFactFallback(fact: AgentDispatchFact, t: TFunction) {
  const rawLabel = fact.label.trim()
  const rawDetail = fact.detail.trim()
  const fallback = getKnownFactFallbackCopy(rawLabel, t)

  return {
    detail: isI18nKeyLeak(rawDetail) ? fallback?.detail : undefined,
    label: isI18nKeyLeak(rawLabel) ? fallback?.label : undefined,
  }
}

function getKnownFactFallbackCopy(label: string, t: TFunction) {
  if (label === "agentChat.thread.dispatchFacts.materialAudit.label") {
    return {
      detail: fallbackTranslation(
        t,
        "agentChat.thread.dispatchPanel.preview.fallbackFactDetail",
        "Task material status synced"
      ),
      label: fallbackTranslation(
        t,
        "agentChat.thread.dispatchPanel.preview.fallbackFactLabel",
        "Task materials"
      ),
    }
  }
  if (label === "agentChat.thread.dispatchFacts.roleAudit.label") {
    return {
      detail: fallbackTranslation(
        t,
        "agentChat.thread.dispatchPanel.preview.fallbackRoleFactDetail",
        "Role status synced"
      ),
      label: fallbackTranslation(
        t,
        "agentChat.thread.dispatchPanel.preview.fallbackRoleFactLabel",
        "Role check"
      ),
    }
  }
  if (label === "agentChat.thread.dispatchFacts.searchAudit.label") {
    return {
      detail: fallbackTranslation(
        t,
        "agentChat.thread.dispatchPanel.preview.fallbackSearchFactDetail",
        "Source status synced"
      ),
      label: fallbackTranslation(
        t,
        "agentChat.thread.dispatchPanel.preview.fallbackSearchFactLabel",
        "Source check"
      ),
    }
  }
  if (label === "agentChat.thread.dispatchFacts.skillAudit.label") {
    return {
      detail: fallbackTranslation(
        t,
        "agentChat.thread.dispatchPanel.preview.fallbackSkillFactDetail",
        "Capability status synced"
      ),
      label: fallbackTranslation(
        t,
        "agentChat.thread.dispatchPanel.preview.fallbackSkillFactLabel",
        "Capability check"
      ),
    }
  }
  return undefined
}

function fallbackTranslation(t: TFunction, key: string, fallback: string) {
  const value = t(key)
  return isI18nKeyLeak(value) ? fallback : value
}

function isI18nKeyLeak(value: string) {
  return /^agentChat\./.test(value)
}

function addFirstLabel(
  selected: Set<number>,
  facts: AgentDispatchFact[],
  label: string
) {
  const index = facts.findIndex((fact) => fact.label === label)
  if (index >= 0) addIndex(selected, index)
}

function addIndex(selected: Set<number>, index: number) {
  if (selected.size < MAX_DISPATCH_FACTS) {
    selected.add(index)
  }
}
