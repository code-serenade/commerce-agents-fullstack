import { BarChart3, Bot, FileText, Megaphone, MonitorCog, PenLine, WalletCards } from "lucide-react"
import type { TFunction } from "i18next"
import type { ComponentType, SVGProps } from "react"
import type { AgentRuntimeEvent, AgentTaskNode, AgentWorkerReport } from "@/api"
export {
  getComputerCards,
  getComputerScreenContext,
} from "./swarm-runtime-view-model"

export type SwarmState =
  | "idle"
  | "queued"
  | "working"
  | "done"
  | "failed"
  | "reworking"
  | "skipped"

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

export interface RuntimeNodeSnapshot {
  attempt?: number
  evaluationReasons: string[]
  report?: AgentWorkerReport
  state: SwarmState
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

export function getRuntimeNodeStates(runtimeEvents: AgentRuntimeEvent[]) {
  const states = new Map<string, RuntimeNodeSnapshot>()
  const ensureNode = (nodeId: string): RuntimeNodeSnapshot => {
    const current = states.get(nodeId)
    if (current) return current
    const next: RuntimeNodeSnapshot = { evaluationReasons: [], state: "queued" }
    states.set(nodeId, next)
    return next
  }

  runtimeEvents.forEach((event) => {
    if (event.type === "task_graph_planned") {
      event.graph.nodes.forEach((node) => (ensureNode(readId(node.id)).state = "queued"))
    } else if (event.type === "task_node_ready") {
      ensureNode(event.nodeId).state = "queued"
    } else if (event.type === "task_node_started") {
      Object.assign(ensureNode(event.nodeId), { state: "working", attempt: event.attempt })
    } else if (event.type === "task_node_reported") {
      Object.assign(ensureNode(event.nodeId), {
        report: event.report,
        state: event.report.status === "failed" ? "failed" : "done",
      })
    } else if (event.type === "task_node_evaluated") {
      const snapshot = ensureNode(readId(event.evaluation.node_id))
      snapshot.evaluationReasons = event.evaluation.reasons
      snapshot.state = event.evaluation.passed ? "done" : snapshot.state
    } else if (event.type === "task_node_rework_requested") {
      Object.assign(ensureNode(event.nodeId), {
        state: "reworking",
        attempt: event.attempt,
        evaluationReasons: [event.instruction],
      })
    } else if (event.type === "task_node_passed") {
      ensureNode(event.nodeId).state = "done"
    } else if (event.type === "task_node_failed") {
      Object.assign(ensureNode(event.nodeId), {
        state: "failed",
        evaluationReasons: [event.reason],
      })
    }
  })
  return states
}

export function getCompletedCount(
  agents: SwarmAgent[],
  runtimeNodeStates: Map<string, RuntimeNodeSnapshot>
) {
  if (runtimeNodeStates.size === 0) return 0
  return agents.filter((agent) => runtimeNodeStates.get(agent.nodeId)?.state === "done").length
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
    .map((event) => formatTimelineEvent(event, agents, t))
    .filter((item): item is NonNullable<typeof item> => item !== null)
    .slice(-8)
}

export function getStatePillLabel(state: SwarmState, t: TFunction) {
  if (state === "done") return t("agentChat.swarm.statePill.done")
  if (state === "working") return t("agentChat.swarm.statePill.working")
  if (state === "queued") return t("agentChat.swarm.statePill.queued")
  if (state === "failed") return t("agentChat.swarm.statePill.failed")
  if (state === "reworking") return t("agentChat.swarm.statePill.reworking")
  if (state === "skipped") return t("agentChat.swarm.statePill.skipped")
  return t("agentChat.swarm.statePill.idle")
}

export function getFallbackResponse(agent: SwarmAgent, state: SwarmState, t: TFunction) {
  if (state === "failed") return t("agentChat.swarm.fallbackResponse.failed")
  if (state === "reworking") return t("agentChat.swarm.fallbackResponse.reworking")
  if (state === "working") return t("agentChat.swarm.fallbackResponse.working", { agent: agent.name })
  if (state === "done") return t("agentChat.swarm.fallbackResponse.done", { agent: agent.name })
  return t("agentChat.swarm.fallbackResponse.idle")
}

export function getDockStateLabel(state: SwarmState, t: TFunction) {
  if (state === "done") return t("agentChat.swarm.dockState.done")
  if (state === "working") return t("agentChat.swarm.dockState.working")
  if (state === "reworking") return t("agentChat.swarm.dockState.reworking")
  if (state === "failed") return t("agentChat.swarm.dockState.failed")
  if (state === "queued") return t("agentChat.swarm.dockState.queued")
  if (state === "skipped") return t("agentChat.swarm.dockState.skipped")
  return t("agentChat.swarm.dockState.idle")
}

export function hasFailedNode(runtimeNodeStates: Map<string, RuntimeNodeSnapshot>) {
  return Array.from(runtimeNodeStates.values()).some((snapshot) => snapshot.state === "failed")
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

function formatTimelineEvent(event: AgentRuntimeEvent, agents: SwarmAgent[], t: TFunction) {
  if (event.type === "commander_thinking") {
    return { label: t("agentChat.swarm.timeline.events.commanderThinking"), tone: "default" as const }
  }
  if (event.type === "task_graph_planned") {
    return { label: t("agentChat.swarm.timeline.events.graphPlanned", { count: event.graph.nodes.length }), tone: "success" as const }
  }
  if (event.type === "task_node_started") {
    return { label: t("agentChat.swarm.timeline.events.nodeStarted", { agent: getAgentName(event.nodeId, agents), attempt: event.attempt }), tone: "default" as const }
  }
  if (event.type === "task_node_rework_requested") {
    return { label: t("agentChat.swarm.timeline.events.nodeRework", { agent: getAgentName(event.nodeId, agents), instruction: event.instruction }), tone: "warning" as const }
  }
  if (event.type === "task_node_passed") {
    return { label: t("agentChat.swarm.timeline.events.nodePassed", { agent: getAgentName(event.nodeId, agents) }), tone: "success" as const }
  }
  if (event.type === "task_node_failed") {
    return { label: t("agentChat.swarm.timeline.events.nodeFailed", { agent: getAgentName(event.nodeId, agents), reason: event.reason }), tone: "error" as const }
  }
  if (event.type === "task_graph_finished") {
    return { label: t("agentChat.swarm.timeline.events.graphFinished"), tone: "success" as const }
  }
  return null
}
