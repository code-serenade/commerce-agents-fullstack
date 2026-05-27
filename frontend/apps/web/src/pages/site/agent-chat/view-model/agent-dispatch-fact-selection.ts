import type { TFunction } from "i18next"
import type { AgentDispatchFact } from "./agent-process-kind-view-model"

const MAX_DISPATCH_FACTS = 6

export function selectDispatchFacts(
  facts: AgentDispatchFact[],
  t: TFunction
) {
  if (facts.length <= MAX_DISPATCH_FACTS) return facts

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
