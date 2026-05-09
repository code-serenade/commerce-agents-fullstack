import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import { readRuntimeNodeId, resolveWorkerNodeId } from "./agent-runtime-event-mapping"
import type { SwarmAgent } from "./swarm-panel-view-model"

export function formatTimelineEvent(
  event: AgentRuntimeEvent,
  agents: SwarmAgent[],
  t: TFunction
) {
  if (event.type === "commander_thinking") {
    return { label: t("agentChat.swarm.timeline.events.commanderThinking"), tone: "default" as const }
  }
  if (event.type === "route_decided") {
    return {
      label: t("agentChat.swarm.timeline.events.routeDecided", {
        mode: event.route_mode,
        role: event.primary_runtime_role ?? "chat",
        strategy: event.strategy_id ?? "default",
      }),
      tone: event.route_mode === "task_graph" ? "success" as const : "default" as const,
    }
  }
  if (event.type === "context_loaded") {
    return {
      label: t("agentChat.swarm.timeline.events.contextLoaded", {
        knowledge: event.knowledge_count ?? 0,
        products: event.product_count ?? event.productIds?.length ?? 0,
      }),
      tone: "success" as const,
    }
  }
  if (event.type === "task_graph_planned") {
    return { label: t("agentChat.swarm.timeline.events.graphPlanned", { count: event.graph.nodes.length }), tone: "success" as const }
  }
  if (event.type === "task_node_started") {
    return { label: t("agentChat.swarm.timeline.events.nodeStarted", { agent: getTimelineAgentName(readRuntimeNodeId(event), agents), attempt: event.attempt }), tone: "default" as const }
  }
  if (event.type === "task_node_rework_requested") {
    return { label: t("agentChat.swarm.timeline.events.nodeRework", { agent: getTimelineAgentName(readRuntimeNodeId(event), agents), instruction: event.instruction }), tone: "warning" as const }
  }
  if (event.type === "task_node_passed") {
    return { label: t("agentChat.swarm.timeline.events.nodePassed", { agent: getTimelineAgentName(readRuntimeNodeId(event), agents) }), tone: "success" as const }
  }
  if (event.type === "task_node_failed") {
    return { label: t("agentChat.swarm.timeline.events.nodeFailed", { agent: getTimelineAgentName(readRuntimeNodeId(event), agents), reason: event.reason }), tone: "error" as const }
  }
  if (event.type === "skill_requested") {
    return {
      label: t("agentChat.swarm.timeline.events.skillRequested", {
        role: getTimelineAgentName(resolveWorkerNodeId(event), agents),
        skill: event.skill_id,
      }),
      tone: "default" as const,
    }
  }
  if (event.type === "skill_result") {
    return {
      label: t("agentChat.swarm.timeline.events.skillResult", {
        skill: event.skill_id,
        status: event.status,
      }),
      tone: getSkillResultTone(event.status),
    }
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

function getSkillResultTone(status: string) {
  if (status === "success") return "success" as const
  if (status === "pending_approval" || status === "deferred") return "warning" as const
  if (status === "failed" || status === "not_found" || status === "validation_error") {
    return "error" as const
  }
  return "default" as const
}

function getTimelineAgentName(nodeId: string, agents: SwarmAgent[]) {
  return agents.find((agent) => agent.nodeId === nodeId)?.name ?? nodeId
}
