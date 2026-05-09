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
  if (event.type === "rag_context_loaded") {
    const reason = event.degradedReason ?? event.degraded_reason
    return {
      label: event.degraded
        ? t("agentChat.swarm.timeline.events.ragContextDegraded", {
            reason: reason ?? "-",
          })
        : t("agentChat.swarm.timeline.events.ragContextLoaded", {
            sources: event.sourceCount ?? event.source_count ?? 0,
          }),
      tone: event.degraded ? "warning" as const : "success" as const,
    }
  }
  if (event.type === "task_graph_planned") {
    return { label: t("agentChat.swarm.timeline.events.graphPlanned", { count: event.graph.nodes.length }), tone: "success" as const }
  }
  if (event.type === "workspace_run_linked") {
    return {
      label: t("agentChat.swarm.timeline.events.workspaceRunLinked", {
        title: event.workspace.title,
      }),
      tone: "success" as const,
    }
  }
  if (event.type === "workspace_run_tasks_linked") {
    return {
      label: t("agentChat.swarm.timeline.events.workspaceTasksLinked", {
        count: event.tasks.length,
      }),
      tone: "success" as const,
    }
  }
  if (event.type === "workspace_run_task_updated") {
    return {
      label: t("agentChat.swarm.timeline.events.workspaceTaskUpdated", {
        status: event.status,
      }),
      tone: event.status === "failed" ? "error" as const : "default" as const,
    }
  }
  if (event.type === "workspace_run_completed") {
    return {
      label: t("agentChat.swarm.timeline.events.workspaceRunCompleted"),
      tone: "success" as const,
    }
  }
  if (event.type === "workspace_artifact_created") {
    return {
      label: t("agentChat.swarm.timeline.events.workspaceArtifactCreated", {
        title: event.title,
      }),
      tone: "success" as const,
    }
  }
  if (event.type === "quality_checked") {
    const quality = event.qualityCheck ?? event.quality_check
    return {
      label: t("agentChat.swarm.timeline.events.qualityChecked", {
        score: Math.round((quality?.score ?? 0) * 100),
        status: quality?.passed
          ? t("agentChat.swarm.timeline.events.qualityPassed")
          : t("agentChat.swarm.timeline.events.qualityNeedsWork"),
      }),
      tone: quality?.passed ? "success" as const : "warning" as const,
    }
  }
  if (event.type === "approval_required") {
    return {
      label: t("agentChat.swarm.timeline.events.approvalRequired", {
        risk: readApprovalRisk(event),
        skill: readApprovalSkill(event),
      }),
      tone: "warning" as const,
    }
  }
  if (event.type === "workspace_run_failed") {
    return {
      label: t("agentChat.swarm.timeline.events.workspaceRunFailed"),
      tone: "error" as const,
    }
  }
  if (event.type === "workspace_run_persistence_error") {
    return {
      label: t("agentChat.swarm.timeline.events.workspacePersistenceError", {
        action: event.action,
      }),
      tone: "warning" as const,
    }
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

function readApprovalSkill(event: Extract<AgentRuntimeEvent, { type: "approval_required" }>) {
  return event.skillId ?? event.skill_id ?? event.approval?.skillName ?? event.approval?.skill_name ?? "-"
}

function readApprovalRisk(event: Extract<AgentRuntimeEvent, { type: "approval_required" }>) {
  return event.riskLevel ?? event.risk_level ?? event.approval?.riskLevel ?? event.approval?.risk_level ?? "-"
}
