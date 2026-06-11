import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  getWorkspaceContextPackItemCount,
  getWorkspaceContextPackReadableCount,
} from "./agent-workspace-context-pack-counts"
import { readRuntimeNodeId, resolveWorkerNodeId } from "./agent-runtime-event-mapping"
import {
  isNonBlockingNodeFailure,
  isNonBlockingWorkspaceFailure,
  isNonBlockingSkillGap,
} from "./thread-boundary-state-view-model"
import { formatUserFacingRuntimeReason } from "./swarm-runtime-reason-view-model"
import type { SwarmAgent } from "./swarm-panel-view-model"

export function formatTimelineEvent(
  event: AgentRuntimeEvent,
  agents: SwarmAgent[],
  runtimeEvents: AgentRuntimeEvent[],
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
  if (event.type === "conversation_context_loaded") {
    return {
      label: t("agentChat.swarm.timeline.events.conversationContextLoaded", {
        recent: event.messageCount ?? event.message_count ?? 0,
        total: event.totalMessageCount ?? event.total_message_count ?? 0,
      }),
      tone: event.readiness === "empty" ? "default" as const : "success" as const,
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
  if (event.type === "workspace_context_loaded") {
    return {
      label: t("agentChat.swarm.timeline.events.workspaceContextLoaded", {
        readiness: event.readiness ?? "empty",
        sources: getWorkspaceContextSourceCount(event),
      }),
      tone: event.readiness === "empty" ? "warning" as const : "success" as const,
    }
  }
  if (event.type === "workspace_context_pack_loaded") {
    const readable = getWorkspaceContextPackReadableCount(event)
    return {
      label: t("agentChat.swarm.timeline.events.workspaceContextPackLoaded", {
        readable,
        total: getWorkspaceContextPackItemCount(event),
      }),
      tone: readable === 0 ? "warning" as const : "success" as const,
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
    const boundary = isBoundaryWorkspaceTaskUpdate(event, runtimeEvents)
    return {
      label: boundary
        ? t("agentChat.swarm.timeline.events.workspaceTaskBoundary", {
            status: event.status,
          })
        : t("agentChat.swarm.timeline.events.workspaceTaskUpdated", {
            status: event.status,
          }),
      tone: boundary
        ? "warning" as const
        : event.status === "failed"
          ? "error" as const
          : "default" as const,
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
  if (event.type === "agent_learning_record_created") {
    return {
      label: t("agentChat.swarm.timeline.events.agentLearningRecordCreated", {
        category: event.category,
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
    if (isNonBlockingWorkspaceFailure(event, runtimeEvents)) {
      return {
        label: t("agentChat.swarm.timeline.events.workspaceTaskBoundary", {
          status: event.status,
        }),
        tone: "warning" as const,
      }
    }
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
    if (isNonBlockingNodeFailure(event, runtimeEvents)) {
      return { label: t("agentChat.swarm.timeline.events.nodeBoundary", { agent: getTimelineAgentName(readRuntimeNodeId(event), agents) }), tone: "warning" as const }
    }
    return {
      label: t("agentChat.swarm.timeline.events.nodeFailed", {
        agent: getTimelineAgentName(readRuntimeNodeId(event), agents),
        reason: formatUserFacingRuntimeReason(event.reason, t),
      }),
      tone: "error" as const,
    }
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
      tone: getSkillResultTone(event, runtimeEvents),
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
    return {
      label: t("agentChat.swarm.timeline.events.failed", {
        reason: formatUserFacingRuntimeReason(event.reason, t),
      }),
      tone: "error" as const,
    }
  }
  if (event.type === "task_graph_finished") {
    return { label: t("agentChat.swarm.timeline.events.graphFinished"), tone: "success" as const }
  }
  return null
}

function getSkillResultTone(
  event: Extract<AgentRuntimeEvent, { type: "skill_result" }>,
  runtimeEvents: AgentRuntimeEvent[]
) {
  if (isNonBlockingSkillGap(event, runtimeEvents)) return "warning" as const
  const status = event.status
  if (status === "success") return "success" as const
  if (status === "pending_approval" || status === "deferred") return "warning" as const
  if (status === "failed" || status === "not_found" || status === "validation_error") {
    return "error" as const
  }
  return "default" as const
}

function isBoundaryWorkspaceTaskUpdate(
  event: Extract<AgentRuntimeEvent, { type: "workspace_run_task_updated" }>,
  runtimeEvents: AgentRuntimeEvent[]
) {
  if (event.status !== "failed") return false
  const metadataText = stringifyUnknown(event.metadata)
  if (metadataText.includes("category_level_boundary")) return true
  const ownerRole = event.ownerRole ?? event.owner_role
  return runtimeEvents.some((item) => {
    if (item.type !== "task_node_failed") return false
    if (!isNonBlockingNodeFailure(item, runtimeEvents)) return false
    return ownerRole ? readRuntimeNodeId(item) === ownerRole : true
  })
}

const getTimelineAgentName = (nodeId: string, agents: SwarmAgent[]) =>
  agents.find((agent) => agent.nodeId === nodeId)?.name ?? nodeId

function readApprovalSkill(event: Extract<AgentRuntimeEvent, { type: "approval_required" }>) {
  return event.skillId ?? event.skill_id ?? event.approval?.skillName ?? event.approval?.skill_name ?? "-"
}

function readApprovalRisk(event: Extract<AgentRuntimeEvent, { type: "approval_required" }>) {
  return event.riskLevel ?? event.risk_level ?? event.approval?.riskLevel ?? event.approval?.risk_level ?? "-"
}

function stringifyUnknown(value: unknown) {
  if (value === null || value === undefined) return ""
  if (typeof value === "string") return value
  try {
    return JSON.stringify(value)
  } catch {
    return ""
  }
}

function getWorkspaceContextSourceCount(event: Extract<AgentRuntimeEvent, { type: "workspace_context_loaded" }>) {
  const explicitCount = event.sourceCount ?? event.source_count
  if (explicitCount !== undefined) return Math.max(explicitCount, 0)
  const counts = event.counts
  if (!counts) return 0
  return (counts.conversationFiles ?? counts.conversation_files ?? 0) + (counts.campaigns ?? 0) +
    (counts.productAssets ?? counts.product_assets ?? 0) + (counts.productMaterials ?? counts.product_materials ?? 0) +
    (counts.workspaceArtifacts ?? counts.workspace_artifacts ?? 0)
}
