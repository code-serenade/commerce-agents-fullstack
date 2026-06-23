import type { AgentRuntimeEvent, AgentWorkerReport } from "@/api"
import { getWorkspaceContextPackReadableCount } from "./agent-workspace-context-pack-counts"
import {
  readRuntimeNodeId,
  resolveWorkerNodeId,
} from "./agent-runtime-event-mapping"
import {
  isNonBlockingWorkspaceFailure,
  isBoundaryEvaluation,
  isBoundaryReport,
  isNonBlockingNodeFailure,
  isNonBlockingSkillGap,
} from "./thread-boundary-state-view-model"

export type SwarmState =
  | "idle"
  | "queued"
  | "working"
  | "done"
  | "failed"
  | "reworking"
  | "skipped"
  | "boundary"

export interface RuntimeNodeSnapshot {
  attempt?: number
  evaluationReasons: string[]
  skillResults: AgentSkillRuntimeResult[]
  report?: AgentWorkerReport
  state: SwarmState
}

export interface AgentSkillRuntimeResult {
  approvalRequired?: boolean | null
  error?: string | null
  executionMode?: string | null
  reason?: string | null
  riskLevel?: string | null
  skillId: string
  status: "requested" | string
  summary?: string | null
  boundary?: boolean
}

export function getRuntimeNodeStates(runtimeEvents: AgentRuntimeEvent[]) {
  const states = new Map<string, RuntimeNodeSnapshot>()
  const ensureNode = (nodeId: string): RuntimeNodeSnapshot => {
    const current = states.get(nodeId)
    if (current) return current
    const next: RuntimeNodeSnapshot = {
      evaluationReasons: [],
      skillResults: [],
      state: "queued",
    }
    states.set(nodeId, next)
    return next
  }

  runtimeEvents.forEach((event) => {
    if (event.type === "task_graph_planned") {
      event.graph.nodes.forEach((node) => {
        ensureNode(readRuntimeId(node.id)).state = "queued"
      })
    } else if (event.type === "route_decided") {
      const primaryRole =
        event.primaryRuntimeRole ?? event.primary_runtime_role ?? "ops"
      const primaryNode = primaryRole === "chat" ? "ops" : primaryRole
      const supportRoles =
        event.supportRuntimeRoles ?? event.support_runtime_roles ?? []
      ensureNode(primaryNode).state =
        event.route_mode === "chat_bypass" ? "working" : "queued"
      supportRoles.forEach((role) => {
        if (role !== "chat") ensureNode(role).state = "queued"
      })
    } else if (
      event.type === "context_loaded" ||
      event.type === "conversation_context_loaded" ||
      event.type === "rag_context_loaded" ||
      event.type === "workspace_context_loaded" ||
      event.type === "workspace_context_pack_loaded"
    ) {
      const degraded =
        (event.type === "rag_context_loaded" && event.degraded) ||
        (event.type === "conversation_context_loaded" && event.readiness === "empty") ||
        (event.type === "workspace_context_loaded" && event.readiness === "empty") ||
        (event.type === "workspace_context_pack_loaded" &&
          (event.readiness === "empty" ||
            getWorkspaceContextPackReadableCount(event) === 0))
      ensureNode("ops").state = degraded ? "reworking" : "working"
    } else if (event.type === "task_node_ready") {
      ensureNode(readRuntimeNodeId(event)).state = "queued"
    } else if (event.type === "task_node_started") {
      Object.assign(ensureNode(readRuntimeNodeId(event)), {
        attempt: event.attempt,
        state: "working",
      })
    } else if (event.type === "task_node_reported") {
      Object.assign(ensureNode(readRuntimeNodeId(event)), {
        report: event.report,
        state: isBoundaryReport(event.report)
          ? "boundary"
          : event.report.status === "failed"
            ? "failed"
            : "done",
      })
    } else if (event.type === "task_node_evaluated") {
      const snapshot = ensureNode(readRuntimeId(event.evaluation.node_id))
      snapshot.evaluationReasons = isBoundaryEvaluation(event.evaluation, runtimeEvents)
        ? []
        : event.evaluation.reasons
      if (event.evaluation.passed && snapshot.state !== "boundary") {
        snapshot.state = "done"
      }
    } else if (event.type === "task_node_rework_requested") {
      Object.assign(ensureNode(readRuntimeNodeId(event)), {
        attempt: event.attempt,
        evaluationReasons: [event.instruction],
        state: "reworking",
      })
    } else if (event.type === "task_node_passed") {
      const snapshot = ensureNode(readRuntimeNodeId(event))
      if (snapshot.state !== "boundary") snapshot.state = "done"
    } else if (event.type === "task_node_failed") {
      const snapshot = ensureNode(readRuntimeNodeId(event))
      if (isNonBlockingNodeFailure(event, runtimeEvents)) {
        if (snapshot.state !== "done") {
          snapshot.state = "boundary"
        }
      } else {
        Object.assign(snapshot, {
          evaluationReasons: [event.reason],
          state: "failed",
        })
      }
    } else if (event.type === "skill_requested") {
      const snapshot = ensureNode(resolveWorkerNodeId(event))
      if (snapshot.state !== "boundary") snapshot.state = "working"
      upsertSkillResult(snapshot, {
        reason: event.reason,
        skillId: event.skill_id,
        status: "requested",
      })
    } else if (event.type === "skill_result") {
      applySkillResult(ensureNode(resolveWorkerNodeId(event)), event, runtimeEvents)
    } else if (event.type === "worker_started") {
      const snapshot = ensureNode(resolveWorkerNodeId(event))
      if (snapshot.state !== "boundary") snapshot.state = "working"
    } else if (event.type === "worker_finished") {
      const snapshot = ensureNode(resolveWorkerNodeId(event))
      if (snapshot.state !== "boundary") snapshot.state = "done"
    } else if (
      event.type === "workspace_artifact_created" ||
      event.type === "agent_learning_record_created" ||
      event.type === "workspace_run_completed" ||
      event.type === "quality_checked"
    ) {
      ensureNode("delivery").state = "done"
    } else if (event.type === "workspace_run_failed") {
      if (!isNonBlockingWorkspaceFailure(event, runtimeEvents)) {
        Object.assign(ensureNode("delivery"), {
          evaluationReasons: [event.summary ?? event.status],
          state: "failed",
        })
      } else {
        ensureNode("delivery").state = "boundary"
      }
    } else if (event.type === "failed") {
      Object.assign(ensureNode("ops"), {
        evaluationReasons: [event.reason],
        state: "failed",
      })
    } else if (isDeliverableCompletionEvent(event)) {
      markDeliverableNodesDone(states)
      ensureNode("delivery").state = "done"
    }
  })

  if (runtimeEvents.some(isDeliverableCompletionEvent)) {
    markDeliverableNodesDone(states)
    ensureNode("delivery").state = "done"
  }

  return states
}

export function getCompletedCount(
  agents: Array<{ nodeId: string }>,
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
) {
  if (runtimeNodeStates.size === 0) return 0
  return agents.filter((agent) => {
    const state = runtimeNodeStates.get(agent.nodeId)?.state
    return state === "done" || state === "boundary"
  }).length
}

export function hasFailedNode(runtimeNodeStates: Map<string, RuntimeNodeSnapshot>) {
  return Array.from(runtimeNodeStates.values()).some((snapshot) => snapshot.state === "failed")
}

function applySkillResult(
  snapshot: RuntimeNodeSnapshot,
  event: Extract<AgentRuntimeEvent, { type: "skill_result" }>,
  runtimeEvents: AgentRuntimeEvent[]
) {
  if (event.status === "failed" || event.status === "not_found" || event.status === "validation_error") {
    if (!isNonBlockingSkillGap(event, runtimeEvents)) {
      snapshot.state = "failed"
    } else if (snapshot.state !== "done") {
      snapshot.state = "boundary"
    }
  } else if (snapshot.state !== "failed") {
    if (snapshot.state !== "boundary") snapshot.state = "working"
  }
  upsertSkillResult(snapshot, {
    approvalRequired: event.approval_required,
    boundary: isNonBlockingSkillGap(event, runtimeEvents),
    error: event.error,
    executionMode: event.execution_mode,
    riskLevel: event.risk_level,
    skillId: event.skill_id,
    status: event.status,
    summary: event.summary,
  })
}

function upsertSkillResult(
  snapshot: RuntimeNodeSnapshot,
  result: AgentSkillRuntimeResult
) {
  const index = snapshot.skillResults.findIndex((item) => item.skillId === result.skillId)
  if (index >= 0) {
    snapshot.skillResults[index] = {
      ...snapshot.skillResults[index],
      ...result,
    }
    return
  }
  snapshot.skillResults.push(result)
}

function markDeliverableNodesDone(states: Map<string, RuntimeNodeSnapshot>) {
  states.forEach((snapshot) => {
    if (snapshot.state === "failed") {
      snapshot.state = "boundary"
    } else if (snapshot.state === "working" || snapshot.state === "queued") {
      snapshot.state = "done"
    }
  })
}

function isDeliverableCompletionEvent(event: AgentRuntimeEvent) {
  return (
    event.type === "output" ||
    event.type === "finished" ||
    event.type === "task_graph_finished" ||
    event.type === "workspace_run_completed" ||
    event.type === "workspace_artifact_created"
  )
}

function readRuntimeId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}
