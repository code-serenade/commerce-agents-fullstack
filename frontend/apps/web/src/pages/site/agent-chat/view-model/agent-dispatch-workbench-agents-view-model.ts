import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  getDispatchPlanPrimaryRole,
  getDispatchPlanSupportRoles,
  getLatestDispatchPlan,
  getLatestGraphNodes,
  getLatestRouteEvent,
  getRoutePrimaryRole,
  getRouteSupportRoles,
  getRuntimeWorkerRoles,
  normalizeRuntimeRole,
} from "./agent-process-runtime-view-model"
import { getInlineAgents } from "./thread-inline-view-model"

export function getWorkbenchAgents(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  t: TFunction
) {
  const agents = getInlineAgents(runtimeEvents, isStreaming, t)
  if (getLatestGraphNodes(runtimeEvents).length > 0) return agents

  const desiredRoles = getRuntimeRelevantRoleIds(runtimeEvents)
  if (desiredRoles.size === 0) {
    const activeAgent = agents.find((agent) => agent.state !== "idle")
    return activeAgent ? [activeAgent] : agents.slice(0, 1)
  }

  const filteredAgents = agents.filter((agent) =>
    desiredRoles.has(normalizeDisplayRole(agent.nodeId))
  )
  return filteredAgents.length ? filteredAgents : agents.slice(0, 1)
}

function getRuntimeRelevantRoleIds(runtimeEvents: AgentRuntimeEvent[]) {
  const roles = new Set<string>()
  const plan = getLatestDispatchPlan(runtimeEvents)
  if (plan) {
    addDisplayRole(roles, getDispatchPlanPrimaryRole(plan))
    getDispatchPlanSupportRoles(plan).forEach((role) => addDisplayRole(roles, role))
  }

  const route = getLatestRouteEvent(runtimeEvents)
  if (route) {
    addDisplayRole(roles, getRoutePrimaryRole(route))
    getRouteSupportRoles(route).forEach((role) => addDisplayRole(roles, role))
  }

  getRuntimeWorkerRoles(runtimeEvents).forEach((role) => addDisplayRole(roles, role))

  if (
    runtimeEvents.some(
      (event) =>
        event.type === "workspace_artifact_created" ||
        event.type === "workspace_run_completed"
    )
  ) {
    roles.add("delivery")
  }

  return roles
}

function addDisplayRole(roles: Set<string>, role: string | null | undefined) {
  roles.add(normalizeDisplayRole(role ?? "ops"))
}

function normalizeDisplayRole(role: string) {
  const normalized = normalizeRuntimeRole(role)
  return normalized === "chat" ? "ops" : normalized
}
