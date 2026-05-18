import type { AgentRuntimeEvent } from "@/api"
import {
  readRuntimeNodeId,
  resolveWorkerNodeId,
} from "./agent-runtime-event-mapping"
import type { InlineAgentState } from "./thread-inline-view-model"
import {
  hasBlockingTaskFailure,
  hasBlockingWorkspaceFailure,
  isBoundaryEvaluation,
  isBoundaryReport,
  isNonBlockingNodeFailure,
  isNonBlockingSkillGap,
  isNonBlockingWorkspaceFailure,
  resolveSkillResultNodeId,
} from "./thread-boundary-state-view-model"

export function getInlineNodeStates(runtimeEvents: AgentRuntimeEvent[]) {
  const states = new Map<string, InlineAgentState>()

  runtimeEvents.forEach((event) => {
    if (event.type === "task_graph_planned") {
      event.graph.nodes.forEach((node) => setQueued(states, readInlineId(node.id)))
    } else if (event.type === "task_node_ready") {
      setQueued(states, readRuntimeNodeId(event))
    } else if (event.type === "task_node_started") {
      setWorking(states, readRuntimeNodeId(event))
    } else if (event.type === "task_node_reported") {
      const state = isBoundaryReport(event.report)
        ? "boundary"
        : event.report.status === "failed"
          ? "failed"
          : "done"
      states.set(readRuntimeNodeId(event), state)
    } else if (event.type === "task_node_evaluated") {
      setEvaluationState(states, event, runtimeEvents)
    } else if (event.type === "task_node_rework_requested") {
      states.set(readRuntimeNodeId(event), "reworking")
    } else if (event.type === "task_node_passed") {
      setDoneUnlessBoundary(states, readRuntimeNodeId(event))
    } else if (event.type === "task_node_failed") {
      setTaskFailureState(states, event, runtimeEvents)
    } else if (event.type === "skill_requested") {
      setWorking(states, resolveWorkerNodeId(event))
    } else if (event.type === "skill_result") {
      setSkillResultState(states, event, runtimeEvents)
    } else if (event.type === "worker_started") {
      setWorking(states, resolveWorkerNodeId(event))
    } else if (event.type === "worker_finished") {
      setDoneUnlessBoundary(states, resolveWorkerNodeId(event))
    } else if (event.type === "workspace_artifact_created") {
      states.set("delivery", "done")
    } else if (event.type === "workspace_run_completed") {
      states.set("delivery", "done")
    } else if (event.type === "workspace_run_failed") {
      states.set(
        "delivery",
        isNonBlockingWorkspaceFailure(event, runtimeEvents)
          ? "boundary"
          : "failed"
      )
    } else if (event.type === "failed") {
      states.set("ops", "failed")
    } else if (event.type === "finished") {
      states.set("delivery", "done")
    }
  })

  return states
}

export function getInlineProcessFailureState(
  runtimeEvents: AgentRuntimeEvent[]
) {
  if (hasBlockingTaskFailure(runtimeEvents)) return "failed"
  if (hasBlockingWorkspaceFailure(runtimeEvents)) return "failed"
  return null
}

export function readInlineId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}

function setQueued(states: Map<string, InlineAgentState>, nodeId: string) {
  states.set(nodeId, "queued")
}

function setWorking(states: Map<string, InlineAgentState>, nodeId: string) {
  if (states.get(nodeId) !== "boundary") states.set(nodeId, "working")
}

function setDoneUnlessBoundary(
  states: Map<string, InlineAgentState>,
  nodeId: string
) {
  if (states.get(nodeId) !== "boundary") states.set(nodeId, "done")
}

function setEvaluationState(
  states: Map<string, InlineAgentState>,
  event: Extract<AgentRuntimeEvent, { type: "task_node_evaluated" }>,
  runtimeEvents: AgentRuntimeEvent[]
) {
  const nodeId = readInlineId(event.evaluation.node_id)
  if (event.evaluation.passed && states.get(nodeId) !== "boundary") {
    states.set(nodeId, "done")
  } else if (isBoundaryEvaluation(event.evaluation, runtimeEvents)) {
    if (states.get(nodeId) !== "done") states.set(nodeId, "boundary")
  }
}

function setTaskFailureState(
  states: Map<string, InlineAgentState>,
  event: Extract<AgentRuntimeEvent, { type: "task_node_failed" }>,
  runtimeEvents: AgentRuntimeEvent[]
) {
  const nodeId = readRuntimeNodeId(event)
  if (isNonBlockingNodeFailure(event, runtimeEvents)) {
    if (states.get(nodeId) !== "done") states.set(nodeId, "boundary")
  } else {
    states.set(nodeId, "failed")
  }
}

function setSkillResultState(
  states: Map<string, InlineAgentState>,
  event: Extract<AgentRuntimeEvent, { type: "skill_result" }>,
  runtimeEvents: AgentRuntimeEvent[]
) {
  const nodeId = resolveSkillResultNodeId(event)
  if (isNonBlockingSkillGap(event, runtimeEvents)) {
    if (states.get(nodeId) !== "done") states.set(nodeId, "boundary")
    return
  }
  states.set(
    nodeId,
    event.status === "failed" ||
      event.status === "not_found" ||
      event.status === "validation_error"
      ? "failed"
      : "working"
  )
}
