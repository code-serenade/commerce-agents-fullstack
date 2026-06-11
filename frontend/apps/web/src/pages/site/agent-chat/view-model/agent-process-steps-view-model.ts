import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { AgentProcessKind } from "./agent-process-kind-view-model"
import {
  getWorkspaceContextPackItemCount,
  getWorkspaceContextPackReadableCount,
} from "./agent-workspace-context-pack-counts"
import {
  getDispatchPlanSteps,
  getLatestGraphNodes,
  getLatestDispatchPlan,
  getLatestRouteEvent,
  getRouteMode,
  getRoutePrimaryRole,
  hasFinished,
} from "./agent-process-runtime-view-model"
import { formatRuntimeRole } from "./agent-process-kind-view-model"

export function getProcessStepRows(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  kind: AgentProcessKind,
  t: TFunction
) {
  const plan = getLatestDispatchPlan(runtimeEvents)
  if (plan) {
    const plannedSteps = getDispatchPlanSteps(plan)
    if (plannedSteps.length > 0) {
      return plannedSteps.slice(0, 8).map((step) => [step.title, step.detail])
    }
  }

  if (kind === "quick_dispatch") {
    return getQuickDispatchStepRows(runtimeEvents, isStreaming, t)
  }

  const graphNodes = getLatestGraphNodes(runtimeEvents)
  if (graphNodes.length > 0) {
    return graphNodes.slice(0, 8).map((node) => [
      node.title || formatRuntimeRole(node.role, t),
      node.objective || t("agentChat.thread.steps.graphNodeReady"),
    ])
  }

  const route = getLatestRouteEvent(runtimeEvents)
  const role = formatRuntimeRole(getRoutePrimaryRole(route), t)
  return [
    [
      t("agentChat.thread.steps.readTarget"),
      t("agentChat.thread.steps.readingRules"),
    ],
    [
      t("agentChat.thread.steps.assignTask"),
      kind === "single_agent"
        ? t("agentChat.thread.steps.singleAgentRoute", { role })
        : t("agentChat.thread.steps.waitingSync"),
    ],
    [
      t("agentChat.thread.steps.enrichMaterial"),
      getSkillStepDetail(runtimeEvents, t),
    ],
  ]
}

function getQuickDispatchStepRows(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  t: TFunction
) {
  const route = getLatestRouteEvent(runtimeEvents)
  const role = formatRuntimeRole(getRoutePrimaryRole(route), t)
  return [
    [
      t("agentChat.thread.steps.readTarget"),
      getContextStepDetail(runtimeEvents, t),
    ],
    [
      t("agentChat.thread.steps.dispatchJudge"),
      route
        ? t("agentChat.thread.steps.dispatchRoute", {
            mode: getRouteMode(route),
            role,
          })
        : t("agentChat.thread.steps.dispatchWaiting"),
    ],
    [
      t("agentChat.thread.steps.runtimeAction"),
      getSkillStepDetail(runtimeEvents, t),
    ],
    [
      t("agentChat.thread.steps.answerReady"),
      hasFinished(runtimeEvents)
        ? t("agentChat.thread.steps.answerFinished")
        : isStreaming
          ? t("agentChat.thread.steps.answerStreaming")
          : t("agentChat.thread.steps.answerSynced"),
    ],
  ]
}

function getContextStepDetail(runtimeEvents: AgentRuntimeEvent[], t: TFunction) {
  const workspace = runtimeEvents.find(
    (event) => event.type === "workspace_context_pack_loaded"
  )
  if (workspace) {
    const readable = getWorkspaceContextPackReadableCount(workspace)
    return t("agentChat.thread.steps.workspaceContext", {
      readable,
      total: getWorkspaceContextPackItemCount(workspace),
    })
  }

  const context = runtimeEvents.find((event) => event.type === "context_loaded")
  if (context) {
    return t("agentChat.thread.steps.contextLoaded", {
      knowledge: context.knowledge_count ?? 0,
      products: context.product_count ?? context.productIds?.length ?? 0,
    })
  }

  const memory = runtimeEvents.find((event) => event.type === "memory_loaded")
  if (memory) {
    return t("agentChat.thread.steps.memoryLoaded", {
      count: memory.count ?? 0,
    })
  }

  return t("agentChat.thread.steps.targetRules")
}

function getSkillStepDetail(runtimeEvents: AgentRuntimeEvent[], t: TFunction) {
  const skillResults = runtimeEvents.filter((event) => event.type === "skill_result")
  if (skillResults.length > 0) {
    const latest = skillResults.at(-1)
    return t("agentChat.thread.steps.skillResult", {
      count: skillResults.length,
      skill: latest?.skill_id ?? "-",
      status: latest?.status ?? "-",
    })
  }

  const skillRequests = runtimeEvents.filter((event) => event.type === "skill_requested")
  if (skillRequests.length > 0) {
    return t("agentChat.thread.steps.skillRequested", {
      count: skillRequests.length,
      skill: skillRequests.at(-1)?.skill_id ?? "-",
    })
  }

  if (
    runtimeEvents.some(
      (event) =>
        event.type === "worker_started" ||
        event.type === "llm_chunk" ||
        event.type === "output"
    )
  ) {
    return t("agentChat.thread.steps.modelRunning")
  }

  return t("agentChat.thread.steps.noExtraSkill")
}
