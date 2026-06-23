import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  getDispatchPlanPrimaryRole,
  getDispatchPlanSkills,
  getDispatchPlanSupportRoles,
  getLatestDispatchPlan,
  getLatestGraphNodes,
  getLatestRouteEvent,
  getRoutePrimaryRole,
  getRouteSupportRoles,
  normalizeRuntimeRole,
} from "./agent-process-runtime-view-model"
import type { AgentDispatchFact } from "./agent-process-kind-view-model"
import {
  mergeSearchEvidencePacketSummaries,
  summarizeSkillOutputEvidence,
} from "./agent-search-evidence-packet-view-model"
import { readInlineId } from "./thread-inline-node-states"

export function appendAuditFacts(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  appendRoleAuditFact(facts, runtimeEvents, t)
  appendSearchAuditFact(facts, runtimeEvents, t)
}

function appendRoleAuditFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const plannedRoles = getPlannedExecutionRoles(runtimeEvents)
  const executedRoles = getExecutedExecutionRoles(runtimeEvents)
  if (plannedRoles.length === 0 && executedRoles.length === 0) return

  const missing = plannedRoles.filter((role) => !executedRoles.includes(role))
  const extra = executedRoles.filter((role) => !plannedRoles.includes(role))
  const tone = missing.length > 0 || extra.length > 0 ? "warning" : "success"
  const coveredRoles = plannedRoles.filter((role) => executedRoles.includes(role))
  const displayRoles = coveredRoles.length > 0 ? coveredRoles : executedRoles

  facts.push({
    detail: t(
      missing.length > 0 || extra.length > 0
        ? "agentChat.thread.dispatchFacts.roleAudit.warning"
        : "agentChat.thread.dispatchFacts.roleAudit.detail",
      {
        executed: formatRoleList(displayRoles, t),
        extra: formatRoleList(extra, t),
        missing: formatRoleList(missing, t),
        planned: formatRoleList(plannedRoles, t),
      }
    ),
    label: t("agentChat.thread.dispatchFacts.roleAudit.label"),
    tone,
  })
}

function appendSearchAuditFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const plannedSkills = getPlannedSearchSkills(runtimeEvents)
  const executedResults = runtimeEvents.filter(isSearchSkillResultEvent)
  const requestedSkills = runtimeEvents.filter(isSearchSkillRequestedEvent)
  const executedSkills = uniqueStrings(executedResults.map((event) => event.skill_id))
  const evidenceSummary = mergeSearchEvidencePacketSummaries(
    executedResults
      .map((event) => summarizeSkillOutputEvidence(event))
      .filter((summary) => summary !== null)
  )
  const attemptedSkills = uniqueStrings([
    ...requestedSkills.map((event) => event.skill_id),
    ...executedSkills,
  ])
  const expectsSearch =
    plannedSkills.length > 0 ||
    getPlannedExecutionRoles(runtimeEvents).includes("search")
  if (!expectsSearch && attemptedSkills.length === 0) return

  const failed = executedResults.filter((event) =>
    ["failed", "not_found", "validation_error"].includes(event.status)
  )
  const tone = executedSkills.length > 0 && failed.length === 0
    ? "success"
    : "warning"

  facts.push({
    detail: t(
      executedSkills.length > 0
        ? "agentChat.thread.dispatchFacts.searchAudit.detail"
        : "agentChat.thread.dispatchFacts.searchAudit.pending",
      {
        claims: evidenceSummary?.claims ?? 0,
        confidence: formatConfidence(evidenceSummary?.confidence ?? null),
        executed: executedSkills.length,
        gaps: evidenceSummary?.gaps ?? 0,
        planned: plannedSkills.length,
        requested: attemptedSkills.length,
        sources: evidenceSummary?.sources ?? 0,
        strength: evidenceSummary?.strength ?? "-",
      }
    ),
    label: t("agentChat.thread.dispatchFacts.searchAudit.label"),
    tone,
  })
}

function getPlannedExecutionRoles(runtimeEvents: AgentRuntimeEvent[]) {
  const graphNodes = getLatestGraphNodes(runtimeEvents)
  const graphRoles = uniqueStrings(
    graphNodes
      .filter((node) => !isFinalSynthesisNode(readInlineId(node.id), node.role))
      .map((node) => normalizeRuntimeRole(node.role))
      .filter(isAuditableRole)
  )
  if (graphRoles.length > 0) return graphRoles

  if (!hasExecutionTrace(runtimeEvents)) return []

  const plan = getLatestDispatchPlan(runtimeEvents)
  if (plan) {
    return uniqueStrings([
      normalizeRuntimeRole(getDispatchPlanPrimaryRole(plan)),
      ...getDispatchPlanSupportRoles(plan).map(normalizeRuntimeRole),
    ].filter(isAuditableRole))
  }

  const route = getLatestRouteEvent(runtimeEvents)
  if (!route) return []
  return uniqueStrings([
    normalizeRuntimeRole(getRoutePrimaryRole(route)),
    ...getRouteSupportRoles(route).map(normalizeRuntimeRole),
  ].filter(isAuditableRole))
}

function getExecutedExecutionRoles(runtimeEvents: AgentRuntimeEvent[]) {
  const roles = new Set<string>()
  runtimeEvents.forEach((event) => {
    if (event.type === "task_node_reported") {
      const role = normalizeRuntimeRole(event.report.role)
      if (!isFinalSynthesisNode(readInlineId(event.report.node_id), role) && isAuditableRole(role)) {
        roles.add(role)
      }
    } else if (event.type === "skill_result") {
      const role = normalizeRuntimeRole(event.role)
      if (isAuditableRole(role)) roles.add(role)
    }
  })
  return Array.from(roles)
}

function getPlannedSearchSkills(runtimeEvents: AgentRuntimeEvent[]) {
  const plan = getLatestDispatchPlan(runtimeEvents)
  if (!plan) return []
  return getDispatchPlanSkills(plan).filter(isSearchSkill)
}

function hasExecutionTrace(runtimeEvents: AgentRuntimeEvent[]) {
  return runtimeEvents.some((event) =>
    [
      "skill_requested",
      "skill_result",
      "task_graph_finished",
      "task_node_failed",
      "task_node_passed",
      "task_node_reported",
      "task_node_started",
      "worker_finished",
      "worker_started",
    ].includes(event.type)
  )
}

function isFinalSynthesisNode(nodeId: string, role: string) {
  const normalizedNodeId = nodeId.trim().toLowerCase()
  return (
    normalizedNodeId === "final" ||
    normalizedNodeId.endsWith("_final") ||
    normalizedNodeId.includes("synthesis") ||
    normalizedNodeId === `${normalizeRuntimeRole(role)}_final`
  )
}

function isAuditableRole(role: string) {
  return role !== "chat" && role !== "delivery"
}

function isSearchSkill(skillId: string) {
  return skillId.trim().toLowerCase().startsWith("search_")
}

function isSearchSkillRequestedEvent(
  event: AgentRuntimeEvent
): event is Extract<AgentRuntimeEvent, { type: "skill_requested" }> {
  return event.type === "skill_requested" && isSearchSkill(event.skill_id)
}

function isSearchSkillResultEvent(
  event: AgentRuntimeEvent
): event is Extract<AgentRuntimeEvent, { type: "skill_result" }> {
  return event.type === "skill_result" && isSearchSkill(event.skill_id)
}

function formatRoleList(roles: string[], t: TFunction) {
  return roles.length > 0
    ? roles.map((role) => t(`agentChat.thread.roleFallbackTitles.${normalizeRuntimeRole(role)}`)).join(", ")
    : "-"
}

function formatConfidence(value: number | null) {
  return typeof value === "number" ? `${Math.round(value * 100)}%` : "-"
}

function uniqueStrings(values: string[]) {
  return Array.from(new Set(values.filter((value) => value.trim().length > 0)))
}
