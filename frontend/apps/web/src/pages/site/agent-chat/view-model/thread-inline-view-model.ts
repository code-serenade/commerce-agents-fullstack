import { FileText, ListChecks, PlugZap, ShieldAlert } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentRuntimeEvent, AgentTaskNode } from "@/api"
import { readRuntimeNodeId, resolveWorkerNodeId } from "./agent-runtime-event-mapping"

export type InlineAgentState =
  | "idle"
  | "queued"
  | "working"
  | "done"
  | "failed"
  | "reworking"

export interface InlineAgent {
  nodeId: string
  name: string
  summary: string
  status: string
  state: InlineAgentState
}

export interface InlineArtifact {
  filename: string
  meta: string
  nodeId: string
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
  const hasGraph = runtimeEvents.some(
    (event) => event.type === "task_graph_planned"
  )
  const hasNodeStarted = runtimeEvents.some(
    (event) => event.type === "task_node_started"
  )

  return [
    [
      t("agentChat.thread.steps.readTarget"),
      hasGraph
        ? t("agentChat.thread.steps.planReady")
        : isStreaming
          ? t("agentChat.thread.steps.readingRules")
          : t("agentChat.thread.steps.targetRules"),
    ],
    [
      t("agentChat.thread.steps.assignTask"),
      hasNodeStarted
        ? t("agentChat.thread.steps.taskRunning")
        : isStreaming
          ? t("agentChat.thread.steps.waitingSync")
          : t("agentChat.thread.steps.checkingConstraints"),
    ],
    [
      t("agentChat.thread.steps.enrichMaterial"),
      hasNodeStarted
        ? t("agentChat.thread.steps.enrichByTask")
        : t("agentChat.thread.steps.enrichFallback"),
    ],
  ]
}

export function getInlineArtifacts(
  agents: InlineAgent[],
  hasRuntime: boolean,
  isStreaming: boolean,
  t: TFunction
): InlineArtifact[] {
  const stateByNode = new Map(
    agents.map((agent) => [agent.nodeId, agent.state])
  )
  const fallbackState = hasRuntime ? "queued" : isStreaming ? "working" : "idle"
  const specs = [
    ["research.dimension.report.md", "MD", "data", t("agentChat.thread.artifactSpecs.dataStatus")],
    ["douyin.ads.launch.plan.xlsx", "XLSX", "douyin_ads", t("agentChat.thread.artifactSpecs.adsStatus")],
    [t("agentChat.thread.artifactSpecs.deliveryFile"), "DOCX", "delivery", t("agentChat.thread.artifactSpecs.deliveryStatus")],
  ] as const

  return specs.map(([filename, meta, nodeId, status]) => ({
    filename,
    meta,
    nodeId,
    status,
    state: stateByNode.get(nodeId) ?? fallbackState,
  }))
}

export function getCommerceHandoffItems(t: TFunction) {
  return [
    { icon: PlugZap, label: t("agentChat.thread.handoff.context.label"), detail: t("agentChat.thread.handoff.context.detail") },
    { icon: ListChecks, label: t("agentChat.thread.handoff.execution.label"), detail: t("agentChat.thread.handoff.execution.detail") },
    { icon: FileText, label: t("agentChat.thread.handoff.delivery.label"), detail: t("agentChat.thread.handoff.delivery.detail") },
    { icon: ShieldAlert, label: t("agentChat.thread.handoff.governance.label"), detail: t("agentChat.thread.handoff.governance.detail") },
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
  if (runtimeEvents.some((event) => event.type === "task_node_failed")) {
    return t("agentChat.thread.processStage.failed")
  }
  if (runtimeEvents.some((event) => event.type === "workspace_run_failed")) {
    return t("agentChat.thread.processStage.failed")
  }
  if (
    runtimeEvents.some(
      (event) => event.type === "workspace_artifact_created" || event.type === "workspace_run_completed"
    )
  ) {
    return t("agentChat.thread.processStage.saved")
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
  if (state === "working" || state === "reworking") {
    return t("agentChat.thread.artifactState.working")
  }
  if (state === "failed") return t("agentChat.thread.artifactState.failed")
  if (state === "queued") return t("agentChat.thread.artifactState.queued")
  return t("agentChat.thread.artifactState.idle")
}

export function shouldLightInlinePixel(state: InlineAgentState, index: number) {
  if (state === "done") return index < 12
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
  const preset = presets.find((agent) => agent.nodeId === nodeId)
  return {
    nodeId,
    name: node.title || preset?.name || node.role,
    summary: preset?.summary || node.objective,
    status: t("agentChat.thread.inlineState.queued"),
    state: "queued",
  }
}

function getInlineNodeStates(runtimeEvents: AgentRuntimeEvent[]) {
  const states = new Map<string, InlineAgentState>()

  runtimeEvents.forEach((event) => {
    if (event.type === "task_graph_planned") {
      event.graph.nodes.forEach((node) =>
        states.set(readInlineId(node.id), "queued")
      )
    } else if (event.type === "task_node_ready") {
      states.set(readRuntimeNodeId(event), "queued")
    } else if (event.type === "task_node_started") {
      states.set(readRuntimeNodeId(event), "working")
    } else if (event.type === "task_node_reported") {
      states.set(readRuntimeNodeId(event), event.report.status === "failed" ? "failed" : "done")
    } else if (event.type === "task_node_evaluated") {
      const nodeId = readInlineId(event.evaluation.node_id)
      if (event.evaluation.passed) states.set(nodeId, "done")
    } else if (event.type === "task_node_rework_requested") {
      states.set(readRuntimeNodeId(event), "reworking")
    } else if (event.type === "task_node_passed") {
      states.set(readRuntimeNodeId(event), "done")
    } else if (event.type === "task_node_failed") {
      states.set(readRuntimeNodeId(event), "failed")
    } else if (event.type === "skill_requested") {
      states.set(resolveWorkerNodeId(event), "working")
    } else if (event.type === "skill_result") {
      states.set(
        resolveWorkerNodeId(event),
        event.status === "failed" || event.status === "not_found" || event.status === "validation_error"
          ? "failed"
          : "working"
      )
    } else if (event.type === "worker_started") {
      states.set(resolveWorkerNodeId(event), "working")
    } else if (event.type === "worker_finished") {
      states.set(resolveWorkerNodeId(event), "done")
    } else if (event.type === "workspace_artifact_created" || event.type === "workspace_run_completed") {
      states.set("delivery", "done")
    } else if (event.type === "workspace_run_failed") {
      states.set("delivery", "failed")
    } else if (event.type === "failed") {
      states.set("ops", "failed")
    } else if (event.type === "finished") {
      states.set("delivery", "done")
    }
  })

  return states
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
  if (state === "failed") return t("agentChat.thread.inlineState.failed")
  if (state === "queued") return t("agentChat.thread.inlineState.queued")
  return t("agentChat.thread.inlineState.idle")
}

function readInlineId(value: string | [string] | { 0: string }) {
  if (typeof value === "string") return value
  if (Array.isArray(value)) return value[0]
  return value[0]
}
