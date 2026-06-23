import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { SwarmAgent } from "./swarm-panel-view-model"
import { isNonBlockingNodeFailure } from "./thread-boundary-state-view-model"
import { formatUserFacingSkillStatus } from "./agent-user-facing-runtime-text"

export type RuntimeNodeEvent = Extract<
  AgentRuntimeEvent,
  {
    type:
      | "task_node_started"
      | "task_node_reported"
      | "task_node_passed"
      | "task_node_failed"
      | "task_node_rework_requested"
      | "skill_requested"
      | "skill_result"
  }
>

export function findLatestNodeEvent(runtimeEvents: AgentRuntimeEvent[]) {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (
      event.type === "task_node_started" ||
      event.type === "task_node_reported" ||
      event.type === "task_node_passed" ||
      event.type === "task_node_failed" ||
      event.type === "task_node_rework_requested" ||
      event.type === "skill_requested" ||
      event.type === "skill_result"
    ) {
      return event
    }
  }
  return null
}

export function formatRuntimeEventDescription(
  event: RuntimeNodeEvent,
  _agents: SwarmAgent[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  if (event.type === "task_node_started") {
    return t("agentChat.swarm.runtimeDescriptions.started", {
      attempt: event.attempt,
    })
  }
  if (event.type === "task_node_reported") {
    return t("agentChat.swarm.runtimeDescriptions.reported")
  }
  if (event.type === "task_node_passed") {
    return t("agentChat.swarm.runtimeDescriptions.passed")
  }
  if (event.type === "task_node_failed") {
    if (isNonBlockingNodeFailure(event, runtimeEvents)) {
      return t("agentChat.swarm.runtimeDescriptions.boundary")
    }
    return t("agentChat.swarm.runtimeDescriptions.failed")
  }
  if (event.type === "skill_requested") {
    return t("agentChat.swarm.runtimeDescriptions.skillRequested")
  }
  if (event.type === "skill_result") {
    return t("agentChat.swarm.runtimeDescriptions.skillResult", {
      status: formatUserFacingSkillStatus(event.status, t),
    })
  }
  return t("agentChat.swarm.runtimeDescriptions.reworking")
}

export function getRuntimeActionLabel(
  event: AgentRuntimeEvent,
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  return t(`agentChat.swarm.runtimeActions.${runtimeActionKey(event, runtimeEvents)}`)
}

function runtimeActionKey(event: AgentRuntimeEvent, runtimeEvents: AgentRuntimeEvent[]) {
  const eventType = event.type
  if (eventType === "commander_thinking") return "commanderThinking"
  if (eventType === "task_graph_planned") return "graphPlanned"
  if (eventType === "task_node_ready") return "nodeReady"
  if (eventType === "task_node_started") return "nodeStarted"
  if (eventType === "task_node_reported") return "nodeReported"
  if (eventType === "task_node_evaluated") return "nodeEvaluated"
  if (eventType === "task_node_rework_requested") return "nodeRework"
  if (eventType === "task_node_passed") return "nodePassed"
  if (eventType === "task_node_failed") {
    return isNonBlockingNodeFailure(event, runtimeEvents)
      ? "nodeBoundary"
      : "nodeFailed"
  }
  if (eventType === "skill_requested") return "skillRequested"
  if (eventType === "skill_result") return "skillResult"
  if (eventType === "task_graph_finished") return "graphFinished"
  return "synced"
}
