import { Bot, FileText, Search, SearchCheck, TerminalSquare } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import { readRuntimeNodeId } from "./agent-runtime-event-mapping"
import {
  compactPreviewLines,
  readReportDocumentLines,
  readReportLines,
} from "./swarm-report-lines-view-model"
import { getContextCard } from "./swarm-context-card-view-model"
import { getSkillCard } from "./swarm-skill-card-view-model"
import { getWorkspacePersistenceCard } from "./swarm-workspace-view-model"
import {
  getDockStateLabel,
  getFallbackResponse,
  getPlannedGraph,
  getAgentName,
  readId,
  type ComputerCard,
  type ComputerScreenContext,
  type RuntimeNodeSnapshot,
  type SwarmAgent,
} from "./swarm-panel-view-model"

export function getComputerCards(input: {
  activeAgent: SwarmAgent
  activeNode?: RuntimeNodeSnapshot
  agents: SwarmAgent[]
  lastTask?: string
  runtimeEvents: AgentRuntimeEvent[]
}, t: TFunction): ComputerCard[] {
  const { activeAgent, activeNode, agents, lastTask, runtimeEvents } = input
  const cards = [getTaskCard(runtimeEvents, lastTask, t)]
  const contextCard = getContextCard(runtimeEvents, t)
  if (contextCard) cards.push(contextCard)
  const workspaceCard = getWorkspacePersistenceCard(runtimeEvents, t)
  if (workspaceCard) cards.push(workspaceCard)
  const latestNodeEvent = findLatestNodeEvent(runtimeEvents)
  cards.push(getRuntimeCard(latestNodeEvent, activeAgent, activeNode, agents, t))
  const skillCard = getSkillCard(activeNode, t)
  if (skillCard) cards.push(skillCard)
  cards.push(getOutputCard(activeAgent, activeNode, t))
  const reworkEvent = findLatestReworkEvent(runtimeEvents)
  if (reworkEvent) {
    cards.push({
      id: "rework",
      title: t("agentChat.swarm.cards.rework.title"),
      meta: getAgentName(readRuntimeNodeId(reworkEvent), agents),
      description: t("agentChat.swarm.cards.rework.description"),
      icon: SearchCheck,
      lines: [reworkEvent.instruction],
      state: "reworking",
    })
  }
  const failedEvent = findLatestFailedEvent(runtimeEvents)
  if (failedEvent) {
    cards.push({
      id: "failed",
      title: t("agentChat.swarm.cards.failed.title"),
      meta: getAgentName(readRuntimeNodeId(failedEvent), agents),
      description: t("agentChat.swarm.cards.failed.description"),
      icon: TerminalSquare,
      lines: [failedEvent.reason],
      state: "failed",
    })
  }
  return cards.slice(-5)
}

export function getComputerScreenContext(
  input: {
    activeAgent: SwarmAgent
    activeNode?: RuntimeNodeSnapshot
    runtimeEvents: AgentRuntimeEvent[]
  },
  t: TFunction
): ComputerScreenContext {
  const { activeAgent, activeNode, runtimeEvents } = input
  const graphNode = getPlannedGraph(runtimeEvents)?.nodes.find(
    (node) => readId(node.id) === activeAgent.nodeId
  )
  const request = graphNode?.objective ?? activeAgent.objective
  const acceptanceLines =
    graphNode?.acceptance.map((item, index) =>
      t("agentChat.swarm.screen.acceptanceLine", { index: index + 1, item })
    ) ?? []
  const lines = compactPreviewLines([
    `# ${activeAgent.name}`,
    t("agentChat.swarm.screen.responsibilityLine", {
      summary: activeAgent.summary,
    }),
    t("agentChat.swarm.screen.objectiveLine", { request }),
    ...acceptanceLines,
    ...readReportDocumentLines(activeNode?.report, t),
    ...(activeNode?.evaluationReasons ?? []).map((reason) =>
      t("agentChat.swarm.screen.reviewLine", { reason })
    ),
  ])

  return {
    filename: getComputerFilename(activeAgent, t),
    lines,
    mode: activeNode?.report
      ? t("agentChat.swarm.screen.reportMode")
      : t("agentChat.swarm.screen.taskMode"),
    request,
  }
}

function getTaskCard(
  runtimeEvents: AgentRuntimeEvent[],
  lastTask: string | undefined,
  t: TFunction
): ComputerCard {
  const graphEvent = runtimeEvents.find(
    (event): event is Extract<AgentRuntimeEvent, { type: "task_graph_planned" }> =>
      event.type === "task_graph_planned"
  )
  if (!graphEvent) {
    return {
      id: "task-input",
      title: t("agentChat.swarm.cards.readTarget.title"),
      meta: t("agentChat.swarm.cards.readTarget.userMeta"),
      description: t("agentChat.swarm.cards.readTarget.inputDescription"),
      icon: FileText,
      lines: [lastTask || t("agentChat.swarm.cards.readTarget.waitingLine")],
      state: "working",
    }
  }
  return {
    id: "graph-planned",
    title: t("agentChat.swarm.cards.readTarget.title"),
    meta: t("agentChat.swarm.cards.readTarget.planMeta"),
    description: t("agentChat.swarm.cards.readTarget.plannedDescription", {
      count: graphEvent.graph.nodes.length,
    }),
    icon: FileText,
    lines: [
      graphEvent.graph.root_task,
      ...graphEvent.graph.nodes
        .slice(0, 3)
        .map((node) => `${node.title}: ${node.objective}`),
    ],
    state: "done",
  }
}

function getRuntimeCard(
  latestNodeEvent: ReturnType<typeof findLatestNodeEvent>,
  activeAgent: SwarmAgent,
  activeNode: RuntimeNodeSnapshot | undefined,
  agents: SwarmAgent[],
  t: TFunction
): ComputerCard {
  return {
    id: "runtime-terminal",
    title: t("agentChat.swarm.cards.assign.title"),
    meta: t("agentChat.swarm.cards.assign.meta"),
    description: latestNodeEvent
      ? formatRuntimeEventDescription(latestNodeEvent, agents, t)
      : t("agentChat.swarm.cards.assign.description"),
    icon: TerminalSquare,
    lines: [
      t("agentChat.swarm.cards.assign.currentTask", { agent: activeAgent.name }),
      t("agentChat.swarm.cards.assign.stateLine", {
        state: getDockStateLabel(activeNode?.state ?? "working", t),
      }),
      latestNodeEvent
        ? t("agentChat.swarm.cards.assign.latestAction", {
            action: getRuntimeActionLabel(latestNodeEvent.type, t),
          })
        : t("agentChat.swarm.cards.assign.latestPlanning"),
    ],
    state: activeNode?.state ?? "working",
  }
}

function getOutputCard(
  activeAgent: SwarmAgent,
  activeNode: RuntimeNodeSnapshot | undefined,
  t: TFunction
): ComputerCard {
  const reportLines = readReportLines(activeNode?.report, t)
  return {
    id: "agent-output",
    title: activeNode?.report
      ? t("agentChat.swarm.cards.output.reportTitle")
      : t("agentChat.swarm.cards.output.materialTitle"),
    meta: activeNode?.report
      ? activeAgent.name
      : t("agentChat.swarm.cards.output.materialMeta"),
    description: activeNode?.report
      ? getFallbackResponse(activeAgent, activeNode.state, t)
      : t("agentChat.swarm.cards.output.description"),
    icon: activeNode?.report ? Bot : Search,
    lines: reportLines.length
      ? reportLines
      : (t("agentChat.swarm.cards.output.fallbackLines", {
          returnObjects: true,
        }) as string[]),
    state: activeNode?.state ?? "queued",
  }
}

function findLatestNodeEvent(runtimeEvents: AgentRuntimeEvent[]) {
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

function findLatestReworkEvent(runtimeEvents: AgentRuntimeEvent[]) {
  return [...runtimeEvents].reverse().find((event) => event.type === "task_node_rework_requested") ?? null
}

function findLatestFailedEvent(runtimeEvents: AgentRuntimeEvent[]) {
  return [...runtimeEvents].reverse().find((event) => event.type === "task_node_failed") ?? null
}

function formatRuntimeEventDescription(
  event: NonNullable<ReturnType<typeof findLatestNodeEvent>>,
  agents: SwarmAgent[],
  t: TFunction
) {
  if (event.type === "task_node_started") {
    return t("agentChat.swarm.runtimeDescriptions.started", {
      agent: getAgentName(readRuntimeNodeId(event), agents),
      attempt: event.attempt,
    })
  }
  if (event.type === "task_node_reported") {
    return t("agentChat.swarm.runtimeDescriptions.reported", {
      agent: getAgentName(readRuntimeNodeId(event), agents),
    })
  }
  if (event.type === "task_node_passed") {
    return t("agentChat.swarm.runtimeDescriptions.passed", {
      agent: getAgentName(readRuntimeNodeId(event), agents),
    })
  }
  if (event.type === "task_node_failed") {
    return t("agentChat.swarm.runtimeDescriptions.failed", {
      agent: getAgentName(readRuntimeNodeId(event), agents),
    })
  }
  if (event.type === "skill_requested") {
    return t("agentChat.swarm.runtimeDescriptions.skillRequested", {
      skill: event.skill_id,
    })
  }
  if (event.type === "skill_result") {
    return t("agentChat.swarm.runtimeDescriptions.skillResult", {
      skill: event.skill_id,
      status: event.status,
    })
  }
  return t("agentChat.swarm.runtimeDescriptions.reworking", {
    agent: getAgentName(readRuntimeNodeId(event), agents),
  })
}

function getRuntimeActionLabel(eventType: AgentRuntimeEvent["type"], t: TFunction) {
  return t(`agentChat.swarm.runtimeActions.${runtimeActionKey(eventType)}`)
}

function runtimeActionKey(eventType: AgentRuntimeEvent["type"]) {
  if (eventType === "commander_thinking") return "commanderThinking"
  if (eventType === "task_graph_planned") return "graphPlanned"
  if (eventType === "task_node_ready") return "nodeReady"
  if (eventType === "task_node_started") return "nodeStarted"
  if (eventType === "task_node_reported") return "nodeReported"
  if (eventType === "task_node_evaluated") return "nodeEvaluated"
  if (eventType === "task_node_rework_requested") return "nodeRework"
  if (eventType === "task_node_passed") return "nodePassed"
  if (eventType === "task_node_failed") return "nodeFailed"
  if (eventType === "skill_requested") return "skillRequested"
  if (eventType === "skill_result") return "skillResult"
  if (eventType === "task_graph_finished") return "graphFinished"
  return "synced"
}

function getComputerFilename(agent: SwarmAgent, t: TFunction) {
  if (agent.nodeId === "data") return "research.dimension.report.md"
  if (agent.nodeId === "accounting") return "budget.risk.assessment.xlsx"
  if (agent.nodeId === "douyin_ads") return "douyin.ads.launch.plan.xlsx"
  if (agent.nodeId === "creative") return "content.script.matrix.md"
  if (agent.nodeId === "ops") return "operation.execution.plan.md"
  if (agent.nodeId === "delivery") {
    return t("agentChat.swarm.screen.deliveryFilename")
  }
  return `${agent.role}.agent.output.md`
}
