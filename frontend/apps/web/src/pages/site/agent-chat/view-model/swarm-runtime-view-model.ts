import { Bot, FileText, Search, SearchCheck, TerminalSquare } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import { readRuntimeNodeId } from "./agent-runtime-event-mapping"
import {
  isNonBlockingNodeFailure,
  isNonBlockingSkillGap,
  resolveSkillResultNodeId,
} from "./thread-boundary-state-view-model"
import {
  findLatestNodeEvent,
  formatRuntimeEventDescription,
  getRuntimeActionLabel,
} from "./swarm-runtime-event-label-view-model"
import { formatUserFacingRuntimeReason } from "./swarm-runtime-reason-view-model"
import { formatTimelineDynamicText } from "./swarm-timeline-user-facing-text"
import { formatUserFacingTaskTitle } from "./swarm-timeline-user-facing-text"
import {
  compactPreviewLines,
  isInternalRuntimeLine,
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
  readId,
  type ComputerCard,
  type ComputerScreenContext,
  type RuntimeNodeSnapshot,
  type SwarmAgent,
} from "./swarm-panel-view-model"
import { hasFinished } from "./agent-process-runtime-view-model"

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
  cards.push(getRuntimeCard(latestNodeEvent, activeAgent, activeNode, agents, runtimeEvents, t))
  const skillCard = getSkillCard(activeNode, t)
  if (skillCard) cards.push(skillCard)
  cards.push(getOutputCard(activeAgent, activeNode, t))
  const reworkEvent = findLatestReworkEvent(runtimeEvents)
  if (reworkEvent) {
    cards.push({
      id: "rework",
      title: t("agentChat.swarm.cards.rework.title"),
      meta: t("agentChat.swarm.cards.rework.meta"),
      description: t("agentChat.swarm.cards.rework.description"),
      icon: SearchCheck,
      lines: [formatTimelineDynamicText(reworkEvent.instruction, t)],
      state: "reworking",
    })
  }
  const boundaryEvent = findLatestBoundaryEvent(runtimeEvents)
  if (boundaryEvent) {
    cards.push({
      id: "boundary",
      title: t("agentChat.swarm.cards.boundary.title"),
      meta: t("agentChat.swarm.cards.boundary.meta"),
      description: t("agentChat.swarm.cards.boundary.description"),
      icon: SearchCheck,
      lines: [t("agentChat.swarm.cards.boundary.line")],
      state: "boundary",
    })
  }
  const failedEvent = findLatestFailedEvent(runtimeEvents)
  if (failedEvent) {
    cards.push({
      id: "failed",
      title: t("agentChat.swarm.cards.failed.title"),
      description: t("agentChat.swarm.cards.failed.description"),
      icon: TerminalSquare,
      lines: [formatUserFacingRuntimeReason(failedEvent.reason, t)],
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
  const request = formatTimelineDynamicText(
    graphNode?.objective ?? activeAgent.objective,
    t
  )
  const acceptanceLines =
    graphNode?.acceptance
      .map((item) => formatTimelineDynamicText(item, t))
      .filter((item) => !isInternalRuntimeLine(item))
      .map((item, index) =>
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
    ...(activeNode?.evaluationReasons ?? [])
      .filter((reason) => !isInternalRuntimeLine(reason))
      .map((reason) => t("agentChat.swarm.screen.reviewLine", { reason })),
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
      formatTimelineDynamicText(graphEvent.graph.root_task, t),
      ...graphEvent.graph.nodes
        .slice(0, 3)
        .map((node) =>
          `${formatUserFacingTaskTitle(node.title, t)}: ${formatTimelineDynamicText(
            node.objective,
            t
          )}`
        ),
    ],
    state: "done",
  }
}

function getRuntimeCard(
  latestNodeEvent: ReturnType<typeof findLatestNodeEvent>,
  activeAgent: SwarmAgent,
  activeNode: RuntimeNodeSnapshot | undefined,
  agents: SwarmAgent[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
): ComputerCard {
  return {
    id: "runtime-terminal",
    title: t("agentChat.swarm.cards.assign.title"),
    meta: t("agentChat.swarm.cards.assign.meta"),
    description: latestNodeEvent
      ? formatRuntimeEventDescription(latestNodeEvent, agents, runtimeEvents, t)
      : t("agentChat.swarm.cards.assign.description"),
    icon: TerminalSquare,
    lines: [
      t("agentChat.swarm.cards.assign.currentTask", { agent: activeAgent.name }),
      t("agentChat.swarm.cards.assign.stateLine", {
        state: getDockStateLabel(activeNode?.state ?? "working", t),
      }),
      latestNodeEvent
        ? t("agentChat.swarm.cards.assign.latestAction", {
            action: getRuntimeActionLabel(latestNodeEvent, runtimeEvents, t),
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

function findLatestReworkEvent(runtimeEvents: AgentRuntimeEvent[]) {
  return [...runtimeEvents].reverse().find((event) => event.type === "task_node_rework_requested") ?? null
}

function findLatestFailedEvent(runtimeEvents: AgentRuntimeEvent[]) {
  if (hasFinished(runtimeEvents)) return null
  return [...runtimeEvents]
    .reverse()
    .find(
      (event): event is Extract<AgentRuntimeEvent, { type: "task_node_failed" }> =>
        event.type === "task_node_failed" &&
        !isNonBlockingNodeFailure(event, runtimeEvents)
    ) ?? null
}

function findLatestBoundaryEvent(runtimeEvents: AgentRuntimeEvent[]) {
  for (const event of [...runtimeEvents].reverse()) {
    if (event.type === "task_node_failed" && isNonBlockingNodeFailure(event, runtimeEvents)) {
      return { nodeId: readRuntimeNodeId(event) }
    }
    if (event.type === "skill_result" && isNonBlockingSkillGap(event, runtimeEvents)) {
      return { nodeId: resolveSkillResultNodeId(event) }
    }
  }
  return null
}

function getComputerFilename(agent: SwarmAgent, t: TFunction) {
  const keyByNodeId: Record<string, string> = {
    accounting: "accountingFilename",
    creative: "creativeFilename",
    data: "dataFilename",
    delivery: "deliveryFilename",
    design: "designFilename",
    engineering: "engineeringFilename",
    ops: "opsFilename",
    search: "searchFilename",
    service: "serviceFilename",
    web: "webFilename",
  }
  const key = keyByNodeId[agent.nodeId] ?? "defaultFilename"
  return t(`agentChat.swarm.screen.${key}`)
}
