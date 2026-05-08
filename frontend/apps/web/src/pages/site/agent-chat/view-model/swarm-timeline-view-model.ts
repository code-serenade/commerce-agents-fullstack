import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import { resolveWorkerNodeId } from "./agent-runtime-event-mapping"
import type { SwarmAgent } from "./swarm-panel-view-model"

export function formatTimelineEvent(
  event: AgentRuntimeEvent,
  agents: SwarmAgent[],
  t: TFunction
) {
  if (event.type === "commander_thinking") {
    return { label: t("agentChat.swarm.timeline.events.commanderThinking"), tone: "default" as const }
  }
  if (event.type === "task_graph_planned") {
    return { label: t("agentChat.swarm.timeline.events.graphPlanned", { count: event.graph.nodes.length }), tone: "success" as const }
  }
  if (event.type === "task_node_started") {
    return { label: t("agentChat.swarm.timeline.events.nodeStarted", { agent: getTimelineAgentName(event.nodeId, agents), attempt: event.attempt }), tone: "default" as const }
  }
  if (event.type === "task_node_rework_requested") {
    return { label: t("agentChat.swarm.timeline.events.nodeRework", { agent: getTimelineAgentName(event.nodeId, agents), instruction: event.instruction }), tone: "warning" as const }
  }
  if (event.type === "task_node_passed") {
    return { label: t("agentChat.swarm.timeline.events.nodePassed", { agent: getTimelineAgentName(event.nodeId, agents) }), tone: "success" as const }
  }
  if (event.type === "task_node_failed") {
    return { label: t("agentChat.swarm.timeline.events.nodeFailed", { agent: getTimelineAgentName(event.nodeId, agents), reason: event.reason }), tone: "error" as const }
  }
  if (event.type === "worker_started") {
    return {
      label: t("agentChat.swarm.timeline.events.workerStarted", {
        agent: getTimelineAgentName(resolveWorkerNodeId(event), agents),
      }),
      tone: "default" as const,
    }
  }
  if (event.type === "worker_finished") {
    return {
      label: t("agentChat.swarm.timeline.events.workerFinished", {
        agent: getTimelineAgentName(resolveWorkerNodeId(event), agents),
      }),
      tone: "success" as const,
    }
  }
  if (event.type === "finished") {
    return { label: t("agentChat.swarm.timeline.events.finished"), tone: "success" as const }
  }
  if (event.type === "failed") {
    return { label: t("agentChat.swarm.timeline.events.failed", { reason: event.reason }), tone: "error" as const }
  }
  if (event.type === "task_graph_finished") {
    return { label: t("agentChat.swarm.timeline.events.graphFinished"), tone: "success" as const }
  }
  return null
}

function getTimelineAgentName(nodeId: string, agents: SwarmAgent[]) {
  return agents.find((agent) => agent.nodeId === nodeId)?.name ?? nodeId
}
