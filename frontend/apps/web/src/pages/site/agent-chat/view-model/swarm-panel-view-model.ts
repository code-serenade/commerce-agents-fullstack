import { BarChart3, Bot, FileText, Megaphone, MonitorCog, PenLine, WalletCards } from "lucide-react"
import type { TFunction } from "i18next"
import type { ComponentType, SVGProps } from "react"
import type { AgentRuntimeEvent, AgentTaskNode } from "@/api"
import { formatTimelineEvent } from "./swarm-timeline-view-model"
export { resolveWorkerNodeId } from "./agent-runtime-event-mapping"
export {
  getCompletedCount,
  getRuntimeNodeStates,
  hasFailedNode,
  type AgentSkillRuntimeResult,
  type RuntimeNodeSnapshot,
  type SwarmState,
} from "./swarm-runtime-state-view-model"
export {
  getComputerCards,
  getComputerScreenContext,
} from "./swarm-runtime-view-model"

import {
  hasFailedNode,
  type RuntimeNodeSnapshot,
  type SwarmState,
} from "./swarm-runtime-state-view-model"

export interface SwarmAgent {
  nodeId: string
  name: string
  role: string
  summary: string
  icon: ComponentType<SVGProps<SVGSVGElement>>
  tone: string
  objective: string
  dependsOn: string[]
}

export interface ComputerCard {
  id: string
  title: string
  meta?: string
  description: string
  icon: ComponentType<SVGProps<SVGSVGElement>>
  lines: string[]
  state: SwarmState
}

export interface ComputerScreenContext {
  filename: string
  mode: string
  request: string
  lines: string[]
}

export const WORKSPACE_LANES = [
  ["agentChat.swarm.lanes.context.label", "agentChat.swarm.lanes.context.value"],
  ["agentChat.swarm.lanes.execution.label", "agentChat.swarm.lanes.execution.value"],
  ["agentChat.swarm.lanes.delivery.label", "agentChat.swarm.lanes.delivery.value"],
  ["agentChat.swarm.lanes.governance.label", "agentChat.swarm.lanes.governance.value"],
] as const

const AGENTS = [
  ["data", "data", BarChart3, "bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-200", []],
  ["accounting", "accounting", WalletCards, "bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-200", ["data"]],
  ["douyin_ads", "douyinAds", Megaphone, "bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-200", ["data"]],
  ["creative", "creative", PenLine, "bg-violet-50 text-violet-700 dark:bg-violet-950/40 dark:text-violet-200", ["data"]],
  ["ops", "ops", MonitorCog, "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200", ["accounting", "douyin_ads", "creative"]],
  ["delivery", "delivery", FileText, "bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-200", ["ops"]],
] as const

const FALLBACK_PHASES = ["data", "parallel", "ops", "delivery"] as const

export function getAgents(graphNodes: AgentTaskNode[], t: TFunction): SwarmAgent[] {
  const presets = AGENTS.map(([nodeId, key, icon, tone, dependsOn]) => ({
    dependsOn: [...dependsOn],
    icon,
    name: t(`agentChat.swarm.agents.${key}.name`),
    nodeId,
    objective: t(`agentChat.swarm.agents.${key}.objective`),
    role: nodeId,
    summary: t(`agentChat.swarm.agents.${key}.summary`),
    tone,
  }))
  if (!graphNodes.length) return presets

  return [
    ...graphNodes.map((node) => {
      const nodeId = readId(node.id)
      const preset = presets.find((agent) => agent.nodeId === nodeId)
      return {
        nodeId,
        name: node.title,
        role: node.role,
        summary: preset?.summary ?? node.objective,
        icon: preset?.icon ?? Bot,
        tone: preset?.tone ?? "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200",
        objective: node.objective,
        dependsOn: node.input_refs.map(readId),
      }
    }),
    presets.find((agent) => agent.nodeId === "delivery") ?? presets.at(-1)!,
  ]
}

export function getPhases(graphNodes: AgentTaskNode[], t: TFunction) {
  if (!graphNodes.length) {
    return FALLBACK_PHASES.map((nodeId) => ({
      nodeId,
      title: t(`agentChat.swarm.phases.${nodeId}.title`),
      detail: t(`agentChat.swarm.phases.${nodeId}.detail`),
    }))
  }
  return [
    ...graphNodes.map((node, index) => ({
      nodeId: readId(node.id),
      title: t("agentChat.swarm.phases.dynamicTitle", {
        index: index + 1,
        title: node.title,
      }),
      detail: node.objective,
    })),
    {
      nodeId: "delivery",
      title: t("agentChat.swarm.phases.dynamicDeliveryTitle", {
        index: graphNodes.length + 1,
      }),
      detail: t("agentChat.swarm.phases.dynamicDeliveryDetail"),
    },
  ]
}

export function getStageLabel(
  input: {
    agents: SwarmAgent[]
    hasTask: boolean
    hasRuntime: boolean
    isStreaming: boolean
    runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
  },
  t: TFunction
) {
  const { agents, hasTask, hasRuntime, isStreaming, runtimeNodeStates } = input
  if (!hasTask) return t("agentChat.swarm.stage.waitingInput")
  if (!hasRuntime) {
    return isStreaming ? t("agentChat.swarm.stage.planning") : t("agentChat.swarm.stage.historyWaiting")
  }
  const failedAgent = agents.find((agent) => runtimeNodeStates.get(agent.nodeId)?.state === "failed")
  if (failedAgent) return t("agentChat.swarm.stage.failedAgent", { agent: failedAgent.name })
  const reworkingAgent = agents.find((agent) => runtimeNodeStates.get(agent.nodeId)?.state === "reworking")
  if (reworkingAgent) return t("agentChat.swarm.stage.reworkingAgent", { agent: reworkingAgent.name })
  const workingAgent = agents.find((agent) => runtimeNodeStates.get(agent.nodeId)?.state === "working")
  if (workingAgent) return t("agentChat.swarm.stage.workingAgent", { agent: workingAgent.name })
  return isStreaming ? t("agentChat.swarm.stage.multiTasking") : t("agentChat.swarm.stage.finished")
}

export function getAgentState(input: {
  agent: SwarmAgent
  fallbackIndex: number
  hasTask: boolean
  isStreaming: boolean
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
}): SwarmState {
  const { agent, fallbackIndex, hasTask, isStreaming, runtimeNodeStates } = input
  const runtimeState = runtimeNodeStates.get(agent.nodeId)?.state
  if (runtimeState) return runtimeState
  if (hasFailedNode(runtimeNodeStates)) return "skipped"
  if (!hasTask || runtimeNodeStates.size === 0) {
    return isStreaming && fallbackIndex === 0 ? "queued" : "idle"
  }
  if (!isStreaming) return "done"
  if (fallbackIndex <= 1) return "done"
  if (fallbackIndex <= 4) return "working"
  return "queued"
}

export function getPlannedGraph(runtimeEvents: AgentRuntimeEvent[]) {
  return [...runtimeEvents].reverse().find((event) => event.type === "task_graph_planned")?.graph
}

export function getTimeline(
  runtimeEvents: AgentRuntimeEvent[],
  agents: SwarmAgent[],
  hasTask: boolean,
  isStreaming: boolean,
  t: TFunction
) {
  if (!runtimeEvents.length) {
    const key = isStreaming ? "planning" : hasTask ? "history" : "empty"
    const count = key === "empty" ? 3 : 2
    return Array.from({ length: count }).map((_, index) => ({
      label: t(`agentChat.swarm.timeline.${key}.${index}`),
      tone: "default" as const,
    }))
  }
  return runtimeEvents
    .map((event) => formatTimelineEvent(event, agents, runtimeEvents, t))
    .filter((item): item is NonNullable<typeof item> => item !== null)
    .slice(-8)
}

export function getStatePillLabel(state: SwarmState, t: TFunction) {
  if (state === "done") return t("agentChat.swarm.statePill.done")
  if (state === "boundary") return t("agentChat.swarm.statePill.boundary")
  if (state === "working") return t("agentChat.swarm.statePill.working")
  if (state === "queued") return t("agentChat.swarm.statePill.queued")
  if (state === "failed") return t("agentChat.swarm.statePill.failed")
  if (state === "reworking") return t("agentChat.swarm.statePill.reworking")
  if (state === "skipped") return t("agentChat.swarm.statePill.skipped")
  return t("agentChat.swarm.statePill.idle")
}

export function getFallbackResponse(agent: SwarmAgent, state: SwarmState, t: TFunction) {
  if (state === "failed") return t("agentChat.swarm.fallbackResponse.failed")
  if (state === "boundary") return t("agentChat.swarm.fallbackResponse.boundary")
  if (state === "reworking") return t("agentChat.swarm.fallbackResponse.reworking")
  if (state === "working") return t("agentChat.swarm.fallbackResponse.working", { agent: agent.name })
  if (state === "done") return t("agentChat.swarm.fallbackResponse.done", { agent: agent.name })
  return t("agentChat.swarm.fallbackResponse.idle")
}

export function getDockStateLabel(state: SwarmState, t: TFunction) {
  if (state === "done") return t("agentChat.swarm.dockState.done")
  if (state === "boundary") return t("agentChat.swarm.dockState.boundary")
  if (state === "working") return t("agentChat.swarm.dockState.working")
  if (state === "reworking") return t("agentChat.swarm.dockState.reworking")
  if (state === "failed") return t("agentChat.swarm.dockState.failed")
  if (state === "queued") return t("agentChat.swarm.dockState.queued")
  if (state === "skipped") return t("agentChat.swarm.dockState.skipped")
  return t("agentChat.swarm.dockState.idle")
}

export const formatAgentNumber = (index: number) =>
  String(Math.max(index + 1, 1)).padStart(2, "0")
export function getAgentName(nodeId: string, agents: SwarmAgent[]) {
  return agents.find((agent) => agent.nodeId === nodeId)?.name ?? nodeId
}

export function readId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}

