import { FileText, ListChecks, PlugZap, SearchCheck } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentRuntimeEvent, AgentTaskNode } from "@/api"
import {
  getInlineNodeStates,
  getInlineProcessFailureState,
  readInlineId,
} from "./thread-inline-node-states"
import { formatRuntimeRole, getProcessStepRows } from "./agent-process-kind-view-model"
import { hasFinished, normalizeRuntimeRole } from "./agent-process-runtime-view-model"
import {
  formatUserFacingStepDetail,
  formatUserFacingStepTitle,
  isInternalRuntimeText,
} from "./agent-user-facing-runtime-text"

export type InlineAgentState =
  | "idle"
  | "queued"
  | "working"
  | "done"
  | "failed"
  | "reworking"
  | "boundary"

export interface InlineAgent {
  nodeId: string
  name: string
  summary: string
  status: string
  state: InlineAgentState
}

const INLINE_AGENT_PRESETS = [
  ["data", "data"],
  ["accounting", "accounting"],
  ["douyin_ads", "douyinAds"],
  ["creative", "creative"],
  ["ops", "ops"],
  ["delivery", "delivery"],
] as const

export function getInlineAgents(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  t: TFunction
): InlineAgent[] {
  const graphNodes = getInlineGraphNodes(runtimeEvents)
  const presets = getInlineAgentPresets(t)
  const baseAgents = graphNodes.length
    ? [
        ...graphNodes.map((node) => mapGraphNodeToInlineAgent(node, presets, t)),
        presets.find((agent) => agent.nodeId === "delivery")!,
      ]
    : presets
  const stateMap = getInlineNodeStates(runtimeEvents)

  return baseAgents.map((agent, index) => {
    const state =
      stateMap.get(agent.nodeId) ?? getFallbackInlineState(index, isStreaming)
    return { ...agent, state, status: getInlineStateLabel(state, t) }
  })
}

export function getInlineSteps(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  t: TFunction
) {
  return getProcessStepRows(runtimeEvents, isStreaming, t)
}

export function getCommerceHandoffItems(t: TFunction) {
  return [
    {
      icon: PlugZap,
      label: t("agentChat.thread.handoff.context.label"),
      detail: t("agentChat.thread.handoff.context.detail"),
    },
    {
      icon: ListChecks,
      label: t("agentChat.thread.handoff.execution.label"),
      detail: t("agentChat.thread.handoff.execution.detail"),
    },
    {
      icon: SearchCheck,
      label: t("agentChat.thread.handoff.review.label"),
      detail: t("agentChat.thread.handoff.review.detail"),
    },
    {
      icon: FileText,
      label: t("agentChat.thread.handoff.delivery.label"),
      detail: t("agentChat.thread.handoff.delivery.detail"),
    },
  ]
}

export function getInlineProcessStage(
  input: {
    hasRuntime: boolean
    isStreaming: boolean
    runtimeEvents: AgentRuntimeEvent[]
  },
  t: TFunction
) {
  const { hasRuntime, isStreaming, runtimeEvents } = input
  if (getInlineProcessFailureState(runtimeEvents) === "failed") {
    return t("agentChat.thread.processStage.failed")
  }
  if (hasFinished(runtimeEvents)) {
    return t("agentChat.thread.processStage.finished")
  }
  if (
    runtimeEvents.some(
      (event) => event.type === "workspace_artifact_created" || event.type === "workspace_run_completed"
    )
  ) {
    return t("agentChat.thread.processStage.finished")
  }
  if (runtimeEvents.some((event) => event.type === "task_graph_finished")) {
    return t("agentChat.thread.processStage.finished")
  }
  if (isStreaming) {
    return hasRuntime
      ? t("agentChat.thread.processStage.running")
      : t("agentChat.thread.processStage.planning")
  }
  return hasRuntime
    ? t("agentChat.thread.processStage.synced")
    : t("agentChat.thread.processStage.history")
}

export function getInlineArtifactStateLabel(
  state: InlineAgentState,
  t: TFunction
) {
  if (state === "done") return t("agentChat.thread.artifactState.done")
  if (state === "boundary") return t("agentChat.thread.artifactState.boundary")
  if (state === "working" || state === "reworking") {
    return t("agentChat.thread.artifactState.working")
  }
  if (state === "failed") return t("agentChat.thread.artifactState.failed")
  if (state === "queued") return t("agentChat.thread.artifactState.queued")
  return t("agentChat.thread.artifactState.idle")
}

export function shouldLightInlinePixel(state: InlineAgentState, index: number) {
  if (state === "done") return index < 12
  if (state === "boundary") return index < 8
  if (state === "working" || state === "reworking") return index < 5
  if (state === "failed") return index < 4
  if (state === "queued") return index < 2
  return false
}

function getInlineAgentPresets(t: TFunction): InlineAgent[] {
  return INLINE_AGENT_PRESETS.map(([nodeId, key]) => ({
    nodeId,
    name: t(`agentChat.thread.agents.${key}.name`),
    summary: t(`agentChat.thread.agents.${key}.summary`),
    status: t("agentChat.thread.agents.standby"),
    state: "idle",
  }))
}

function getInlineGraphNodes(runtimeEvents: AgentRuntimeEvent[]): AgentTaskNode[] {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === "task_graph_planned") return event.graph.nodes
  }
  return []
}

export function isChatOnlyRuntime(runtimeEvents: AgentRuntimeEvent[]) {
  if (
    runtimeEvents.some(
      (event) =>
        event.type === "route_decided" && event.route_mode === "chat_bypass"
    )
  ) {
    return true
  }

  const graphNodes = getInlineGraphNodes(runtimeEvents)
  if (graphNodes.length !== 1) return false

  const [node] = graphNodes
  return node.role === "chat" || readInlineId(node.id) === "chat"
}

function mapGraphNodeToInlineAgent(
  node: AgentTaskNode,
  presets: InlineAgent[],
  t: TFunction
): InlineAgent {
  const nodeId = readInlineId(node.id)
  const role = normalizeRuntimeRole(node.role || nodeId)
  const preset =
    presets.find((agent) => agent.nodeId === nodeId) ||
    presets.find((agent) => agent.nodeId === role)
  return {
    nodeId,
    name: preset?.name || formatUserFacingNodeTitle(node.title || node.role, t),
    summary:
      preset?.summary ||
      formatUserFacingStepDetail(
        node.objective || formatRuntimeRole(node.role, t),
        t
      ),
    status: t("agentChat.thread.inlineState.queued"),
    state: "queued",
  }
}

function formatUserFacingNodeTitle(title: string, t: TFunction) {
  const trimmed = title.trim()
  if (!trimmed || isInternalRuntimeText(trimmed)) {
    return t("agentChat.thread.steps.userFacingTitles.generic")
  }
  return formatUserFacingStepTitle(trimmed, t)
}

function getFallbackInlineState(index: number, isStreaming: boolean): InlineAgentState {
  if (!isStreaming) return "idle"
  if (index === 0) return "working"
  if (index <= 3) return "queued"
  return "idle"
}

function getInlineStateLabel(state: InlineAgentState, t: TFunction) {
  if (state === "done") return t("agentChat.thread.inlineState.done")
  if (state === "working") return t("agentChat.thread.inlineState.working")
  if (state === "reworking") return t("agentChat.thread.inlineState.reworking")
  if (state === "boundary") return t("agentChat.thread.inlineState.boundary")
  if (state === "failed") return t("agentChat.thread.inlineState.failed")
  if (state === "queued") return t("agentChat.thread.inlineState.queued")
  return t("agentChat.thread.inlineState.idle")
}

