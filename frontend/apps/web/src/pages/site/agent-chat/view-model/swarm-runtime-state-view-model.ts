import type { AgentRuntimeEvent, AgentWorkerReport } from "@/api"
import {
  readRuntimeNodeId,
  resolveWorkerNodeId,
} from "./agent-runtime-event-mapping"

export type SwarmState =
  | "idle"
  | "queued"
  | "working"
  | "done"
  | "failed"
  | "reworking"
  | "skipped"

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
    } else if (
      event.type === "context_loaded" ||
      event.type === "rag_context_loaded" ||
      event.type === "workspace_context_loaded"
    ) {
      const degraded =
        (event.type === "rag_context_loaded" && event.degraded) ||
        (event.type === "workspace_context_loaded" && event.readiness === "empty")
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
        state: event.report.status === "failed" ? "failed" : "done",
      })
    } else if (event.type === "task_node_evaluated") {
      const snapshot = ensureNode(readRuntimeId(event.evaluation.node_id))
      snapshot.evaluationReasons = event.evaluation.reasons
      snapshot.state = event.evaluation.passed ? "done" : snapshot.state
    } else if (event.type === "task_node_rework_requested") {
      Object.assign(ensureNode(readRuntimeNodeId(event)), {
        attempt: event.attempt,
        evaluationReasons: [event.instruction],
        state: "reworking",
      })
    } else if (event.type === "task_node_passed") {
      ensureNode(readRuntimeNodeId(event)).state = "done"
    } else if (event.type === "task_node_failed") {
      Object.assign(ensureNode(readRuntimeNodeId(event)), {
        evaluationReasons: [event.reason],
        state: "failed",
      })
    } else if (event.type === "skill_requested") {
      const snapshot = ensureNode(resolveWorkerNodeId(event))
      snapshot.state = "working"
      upsertSkillResult(snapshot, {
        reason: event.reason,
        skillId: event.skill_id,
        status: "requested",
      })
    } else if (event.type === "skill_result") {
      applySkillResult(ensureNode(resolveWorkerNodeId(event)), event)
    } else if (event.type === "worker_started") {
      ensureNode(resolveWorkerNodeId(event)).state = "working"
    } else if (event.type === "worker_finished") {
      ensureNode(resolveWorkerNodeId(event)).state = "done"
    } else if (
      event.type === "workspace_artifact_created" ||
      event.type === "workspace_run_completed" ||
      event.type === "quality_checked"
    ) {
      ensureNode("delivery").state = "done"
    } else if (event.type === "workspace_run_failed") {
      Object.assign(ensureNode("delivery"), {
        evaluationReasons: [event.summary ?? event.status],
        state: "failed",
      })
    } else if (event.type === "failed") {
      Object.assign(ensureNode("ops"), {
        evaluationReasons: [event.reason],
        state: "failed",
      })
    } else if (event.type === "finished") {
      ensureNode("delivery").state = "done"
    }
  })

  return states
}

export function getCompletedCount(
  agents: Array<{ nodeId: string }>,
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
) {
  if (runtimeNodeStates.size === 0) return 0
  return agents.filter((agent) => runtimeNodeStates.get(agent.nodeId)?.state === "done").length
}

export function hasFailedNode(runtimeNodeStates: Map<string, RuntimeNodeSnapshot>) {
  return Array.from(runtimeNodeStates.values()).some((snapshot) => snapshot.state === "failed")
}

function applySkillResult(
  snapshot: RuntimeNodeSnapshot,
  event: Extract<AgentRuntimeEvent, { type: "skill_result" }>
) {
  if (event.status === "failed" || event.status === "not_found" || event.status === "validation_error") {
    snapshot.state = "failed"
  } else if (snapshot.state !== "failed") {
    snapshot.state = "working"
  }
  upsertSkillResult(snapshot, {
    approvalRequired: event.approval_required,
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

function readRuntimeId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}
