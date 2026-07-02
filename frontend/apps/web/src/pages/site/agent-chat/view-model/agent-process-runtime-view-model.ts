import type { AgentRuntimeEvent, AgentTaskNode } from "@/api"
import { readInlineId } from "./thread-inline-node-states"

export function getLatestGraphNodes(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "task_graph_planned") return event.graph.nodes
  }
  return []
}

export function getLatestRouteEvent(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "route_decided") return event
  }
  return null
}

export function getLatestDispatchPlan(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "dispatch_planned") return event
  }
  return null
}

export function getRouteMode(
  event: Extract<AgentRuntimeEvent, { type: "route_decided" }>
) {
  return event.routeMode ?? event.route_mode
}

export function getRoutePrimaryRole(
  event: Extract<AgentRuntimeEvent, { type: "route_decided" }> | null
) {
  return event?.primaryRuntimeRole ?? event?.primary_runtime_role ?? "chat"
}

export function getRouteSupportRoles(
  event: Extract<AgentRuntimeEvent, { type: "route_decided" }>
) {
  return event.supportRuntimeRoles ?? event.support_runtime_roles ?? []
}

export function hasFinished(runtimeEvents: AgentRuntimeEvent[]) {
  return runtimeEvents.some(
    (event) =>
      event.type === "output" ||
      event.type === "finished" ||
      event.type === "task_graph_finished" ||
      event.type === "workspace_run_completed" ||
      event.type === "workspace_artifact_created"
  )
}

export function isChatNode(node: AgentTaskNode) {
  return normalizeRuntimeRole(node.role) === "chat" || readInlineId(node.id) === "chat"
}

export function isQuickDispatchRuntime(runtimeEvents: AgentRuntimeEvent[]) {
  const plan = getLatestDispatchPlan(runtimeEvents)
  if (plan && getDispatchPlanRouteMode(plan) === "chat_bypass") return true

  const route = getLatestRouteEvent(runtimeEvents)
  if (route && getRouteMode(route) === "chat_bypass") return true

  const graphNodes = getLatestGraphNodes(runtimeEvents)
  if (graphNodes.length !== 1) return false
  return isChatNode(graphNodes[0]!)
}

export function getDispatchPlanRouteMode(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.routeMode ?? event.route_mode
}

export function getDispatchPlanPrimaryRole(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.primaryRuntimeRole ?? event.primary_runtime_role ?? "chat"
}

export function getDispatchPlanSupportRoles(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.supportRuntimeRoles ?? event.support_runtime_roles ?? []
}

export function getDispatchPlanSteps(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.plannedSteps ?? event.planned_steps ?? []
}

export function getDispatchPlanSkills(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.selectedSkills ?? event.selected_skills ?? []
}

export function getDispatchPlanStudentExplanation(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.studentVisibleExplanation ?? event.student_visible_explanation ?? ""
}

export function getDispatchPlanReasoningSummary(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.reasoningSummary ?? event.reasoning_summary ?? ""
}

export function getDispatchPlanContextBudget(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.contextBudget ?? event.context_budget ?? null
}

export function getDispatchPlanAdaptiveProfile(
  event: Extract<AgentRuntimeEvent, { type: "dispatch_planned" }>
) {
  return event.adaptiveProfile ?? event.adaptive_profile ?? null
}

export function getRuntimeWorkerRoles(runtimeEvents: AgentRuntimeEvent[]) {
  const roles = new Set<string>()
  runtimeEvents.forEach((event) => {
    if (
      event.type === "skill_requested" ||
      event.type === "skill_result" ||
      event.type === "worker_started" ||
      event.type === "worker_finished"
    ) {
      roles.add(resolveProcessWorkerRole(event))
    }
  })
  roles.delete("delivery")
  const route = getLatestRouteEvent(runtimeEvents)
  const primaryRole = normalizeRuntimeRole(getRoutePrimaryRole(route))
  if (primaryRole && primaryRole !== "chat") roles.add(primaryRole)
  return Array.from(roles)
}

export function normalizeRuntimeRole(role: string) {
  const normalized = role.trim().toLowerCase()
  if (!normalized) return "chat"
  if (normalized.includes("chat")) return "chat"
  if (
    normalized.includes("douyin") ||
    normalized.includes("qianchuan") ||
    normalized.includes("ads")
  ) {
    return "ops"
  }
  if (normalized.includes("account")) return "accounting"
  if (normalized.includes("creative")) return "creative"
  if (normalized.includes("delivery")) return "delivery"
  if (normalized.includes("ops") || normalized.includes("operation")) return "ops"
  if (normalized.includes("service")) return "service"
  if (normalized.includes("design")) return "design"
  if (normalized.includes("engineer")) return "engineering"
  if (normalized.includes("search")) return "search"
  if (normalized.includes("data")) return "data"
  if (normalized.includes("web") || normalized.includes("seo")) return "web"
  return normalized
}

function resolveProcessWorkerRole(
  event: Extract<
    AgentRuntimeEvent,
    { type: "skill_requested" | "skill_result" | "worker_started" | "worker_finished" }
  >
) {
  if ("role" in event && typeof event.role === "string") {
    return normalizeRuntimeRole(event.role)
  }
  return normalizeRuntimeRole(event.workerId ?? event.worker_id ?? "chat")
}
