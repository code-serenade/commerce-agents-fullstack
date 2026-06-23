import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import { getInlineProcessFailureState } from "./thread-inline-node-states"
import {
  getLatestGraphNodes,
  getLatestDispatchPlan,
  getDispatchPlanPrimaryRole,
  getDispatchPlanRouteMode,
  getDispatchPlanStudentExplanation,
  getDispatchPlanSupportRoles,
  getLatestRouteEvent,
  getRoutePrimaryRole,
  getRouteSupportRoles,
  getRuntimeWorkerRoles,
  hasFinished,
  isChatNode,
  isQuickDispatchRuntime,
  normalizeRuntimeRole,
} from "./agent-process-runtime-view-model"
import { getProcessStepRows as getProcessStepRowsByKind } from "./agent-process-steps-view-model"

export { getDispatchFacts } from "./agent-process-facts-view-model"

export type AgentProcessKind =
  | "quick_dispatch"
  | "single_agent"
  | "multi_agent"

export interface AgentProcessProgress {
  completed: number
  label: string
  percent: number
  stage: string
  total: number
}

export interface AgentDispatchFact {
  detail: string
  label: string
  tone: "default" | "success" | "warning" | "error"
}

export function getAgentProcessKind(
  runtimeEvents: AgentRuntimeEvent[]
): AgentProcessKind {
  const plan = getLatestDispatchPlan(runtimeEvents)
  if (plan) {
    const supportRoles = getDispatchPlanSupportRoles(plan).filter(
      (role) => normalizeRuntimeRole(role) !== "chat"
    )
    if (getDispatchPlanRouteMode(plan) === "chat_bypass") return "quick_dispatch"
    if (supportRoles.length > 0) return "multi_agent"
    if (normalizeRuntimeRole(getDispatchPlanPrimaryRole(plan)) !== "chat") {
      return "single_agent"
    }
  }

  if (isQuickDispatchRuntime(runtimeEvents)) return "quick_dispatch"

  const graphNodes = getLatestGraphNodes(runtimeEvents)
  const nonChatGraphNodes = graphNodes.filter((node) => !isChatNode(node))
  if (nonChatGraphNodes.length > 1) return "multi_agent"
  if (nonChatGraphNodes.length === 1) return "single_agent"

  const route = getLatestRouteEvent(runtimeEvents)
  if (route) {
    const supportRoles = getRouteSupportRoles(route).filter(
      (role) => normalizeRuntimeRole(role) !== "chat"
    )
    if (supportRoles.length > 0) return "multi_agent"
    if (normalizeRuntimeRole(getRoutePrimaryRole(route)) !== "chat") {
      return "single_agent"
    }
  }

  const workerRoles = getRuntimeWorkerRoles(runtimeEvents)
  if (workerRoles.length > 1) return "multi_agent"
  if (workerRoles.length === 1 && workerRoles[0] !== "chat") {
    return "single_agent"
  }

  return "quick_dispatch"
}

export function getAgentProcessTitle(
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  return t(`agentChat.thread.processKinds.${getAgentProcessKind(runtimeEvents)}.title`)
}

export function getAgentProcessBadge(
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  if (hasFinished(runtimeEvents)) {
    return t("agentChat.thread.processKinds.completed.badge")
  }
  return t(`agentChat.thread.processKinds.${getAgentProcessKind(runtimeEvents)}.badge`)
}

export function shouldShowProcessAgentGrid(kind: AgentProcessKind) {
  return kind !== "quick_dispatch"
}

export function getAgentProcessSummary(
  runtimeEvents: AgentRuntimeEvent[],
  task: string,
  t: TFunction
) {
  const kind = getAgentProcessKind(runtimeEvents)
  const plan = getLatestDispatchPlan(runtimeEvents)
  if (plan) {
    const explanation = getDispatchPlanStudentExplanation(plan)
    const readableExplanation = formatStudentVisibleExplanation(explanation, t)
    if (readableExplanation) return readableExplanation
  }
  const route = getLatestRouteEvent(runtimeEvents)
  if (kind === "quick_dispatch") {
    return t("agentChat.thread.processKinds.quick_dispatch.summary", {
      role: formatRuntimeRole(getRoutePrimaryRole(route), t),
      task,
    })
  }
  if (kind === "single_agent") {
    return t("agentChat.thread.processKinds.single_agent.summary", {
      role: formatRuntimeRole(getRoutePrimaryRole(route), t),
      task,
    })
  }
  return t("agentChat.thread.processKinds.multi_agent.summary", { task })
}

export function getAgentProcessProgress(
  input: {
    completedCount?: number
    isStreaming: boolean
    runtimeEvents: AgentRuntimeEvent[]
    totalCount?: number
  },
  t: TFunction
): AgentProcessProgress {
  const { completedCount, isStreaming, runtimeEvents, totalCount } = input
  const kind = getAgentProcessKind(runtimeEvents)
  const stage = getProcessStage(runtimeEvents, isStreaming, kind, t)

  if (kind === "quick_dispatch") {
    const completed = getQuickDispatchCompletedCount(runtimeEvents)
    const total = 4
    return {
      completed,
      label: t("agentChat.thread.processKinds.quick_dispatch.progress", {
        completed,
        total,
      }),
      percent: Math.min(100, Math.round((completed / total) * 100)),
      stage,
      total,
    }
  }

  const total = Math.max(totalCount ?? 0, 1)
  const completed = hasFinished(runtimeEvents)
    ? total
    : Math.min(completedCount ?? 0, total)
  return {
    completed,
    label: t(`agentChat.thread.processKinds.${kind}.progress`, {
      completed,
      total,
    }),
    percent: Math.min(100, Math.round((completed / total) * 100)),
    stage,
    total,
  }
}

export function getProcessStepRows(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  t: TFunction
) {
  return getProcessStepRowsByKind(
    runtimeEvents,
    isStreaming,
    getAgentProcessKind(runtimeEvents),
    t
  )
}

export function formatRuntimeRole(role: string | null | undefined, t: TFunction) {
  const normalized = normalizeRuntimeRole(role ?? "chat")
  return t(`agentChat.thread.roleFallbackTitles.${normalized}`)
}

function getProcessStage(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  kind: AgentProcessKind,
  t: TFunction
) {
  if (hasFinished(runtimeEvents)) {
    return t(`agentChat.thread.processKinds.${kind}.stageFinished`)
  }
  if (getInlineProcessFailureState(runtimeEvents) === "failed") {
    return t("agentChat.thread.processStage.failed")
  }
  if (
    runtimeEvents.some(
      (event) =>
        event.type === "failed" ||
        event.type === "workspace_run_failed"
    )
  ) {
    return t("agentChat.thread.processStage.failed")
  }
  if (isStreaming) return t(`agentChat.thread.processKinds.${kind}.stageRunning`)
  return t(`agentChat.thread.processKinds.${kind}.stageSynced`)
}

function getQuickDispatchCompletedCount(runtimeEvents: AgentRuntimeEvent[]) {
  if (hasFinished(runtimeEvents)) return 4
  const milestones = [
    runtimeEvents.length > 0,
    runtimeEvents.some(
      (event) =>
        event.type === "memory_loaded" ||
        event.type === "context_loaded" ||
        event.type === "conversation_context_loaded" ||
        event.type === "rag_context_loaded" ||
        event.type === "workspace_context_loaded" ||
        event.type === "workspace_context_pack_loaded"
    ),
    Boolean(getLatestRouteEvent(runtimeEvents)),
    runtimeEvents.some(
      (event) =>
        event.type === "worker_started" ||
        event.type === "skill_requested" ||
        event.type === "skill_result" ||
        event.type === "llm_chunk" ||
        event.type === "output"
    ),
  ]
  return Math.max(1, milestones.filter(Boolean).length)
}

function formatStudentVisibleExplanation(explanation: string, t: TFunction) {
  const trimmed = explanation.trim()
  if (!trimmed) return ""

  return trimmed
    .replace(
      /这个问题会拆给\s*\d+\s*个岗位\/步骤协作，过程更长但结果会更完整。?/g,
      t("agentChat.thread.processKinds.sanitized.dispatchCollaboration")
    )
    .replace(
      /岗位\/步骤协作|岗位协作/g,
      t("agentChat.thread.processKinds.sanitized.multiCheck")
    )
    .replace(
      /多个参考视角/g,
      t("agentChat.thread.processKinds.sanitized.multiReference")
    )
}
