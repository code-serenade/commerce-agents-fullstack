import { Archive, CheckCircle2, Database, FileCheck2, FileText, ShieldAlert, ShieldCheck, TriangleAlert } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { ComputerCard } from "./swarm-panel-view-model"

export function getWorkspacePersistenceCard(
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
): ComputerCard | null {
  const linked = findLatest(runtimeEvents, "workspace_run_linked")
  const taskUpdate = findLatest(runtimeEvents, "workspace_run_task_updated")
  const tasksLinked = findLatest(runtimeEvents, "workspace_run_tasks_linked")
  const completed = findLatest(runtimeEvents, "workspace_run_completed")
  const artifactCreated = findLatest(runtimeEvents, "workspace_artifact_created")
  const qualityChecked = findLatest(runtimeEvents, "quality_checked")
  const approvalRequired = findLatest(runtimeEvents, "approval_required")
  const failed = findLatest(runtimeEvents, "workspace_run_failed")
  const persistenceError = findLatest(runtimeEvents, "workspace_run_persistence_error")

  if (persistenceError) {
    return {
      id: "workspace-persistence-error",
      title: t("agentChat.swarm.cards.workspace.errorTitle"),
      meta: persistenceError.action,
      description: t("agentChat.swarm.cards.workspace.errorDescription"),
      icon: TriangleAlert,
      lines: [persistenceError.message],
      state: "failed",
    }
  }

  if (failed) {
    return {
      id: "workspace-run-failed",
      title: t("agentChat.swarm.cards.workspace.failedTitle"),
      meta: readWorkspaceRunId(failed),
      description: t("agentChat.swarm.cards.workspace.failedDescription"),
      icon: TriangleAlert,
      lines: readWorkspaceRunLines(failed, t),
      state: "failed",
    }
  }

  if (approvalRequired) {
    const approval = approvalRequired.approval
    const alert = approvalRequired.alert
    const skill =
      approvalRequired.skillId ??
      approvalRequired.skill_id ??
      approval?.skillName ??
      approval?.skill_name ??
      "-"
    const risk =
      approvalRequired.riskLevel ??
      approvalRequired.risk_level ??
      approval?.riskLevel ??
      approval?.risk_level ??
      "-"
    return {
      id: "workspace-approval-required",
      title: t("agentChat.swarm.cards.workspace.approvalRequiredTitle"),
      meta: t("agentChat.swarm.cards.workspace.approvalRequiredMeta", {
        risk,
        status: approval?.status ?? "pending",
      }),
      description: t("agentChat.swarm.cards.workspace.approvalRequiredDescription"),
      icon: ShieldAlert,
      lines: [
        t("agentChat.swarm.cards.workspace.approvalSkillLine", { skill }),
        t("agentChat.swarm.cards.workspace.approvalLine", {
          id: approval?.id?.slice(0, 8) ?? "-",
          status: approval?.status ?? "pending",
        }),
        t("agentChat.swarm.cards.workspace.approvalReasonLine", {
          reason: approvalRequired.reason ?? approval?.reason ?? "-",
        }),
        t("agentChat.swarm.cards.workspace.approvalAlertLine", {
          message: alert?.message ?? "-",
        }),
      ],
      state: "reworking",
    }
  }

  if (qualityChecked) {
    const quality = qualityChecked.qualityCheck ?? qualityChecked.quality_check
    const trust = qualityChecked.trustUpdate ?? qualityChecked.trust_update
    const records = qualityChecked.learningRecords ?? qualityChecked.learning_records ?? []
    return {
      id: "workspace-quality-checked",
      title: t("agentChat.swarm.cards.workspace.qualityCheckedTitle"),
      meta: quality?.passed
        ? t("agentChat.swarm.cards.workspace.qualityPassedMeta")
        : t("agentChat.swarm.cards.workspace.qualityNeedsWorkMeta"),
      description: t("agentChat.swarm.cards.workspace.qualityCheckedDescription"),
      icon: ShieldCheck,
      lines: [
        t("agentChat.swarm.cards.workspace.qualityScoreLine", {
          score: Math.round((quality?.score ?? 0) * 100),
        }),
        t("agentChat.swarm.cards.workspace.trustScoreLine", {
          level: trust?.newLevel ?? trust?.new_level ?? "-",
          score: Math.round((trust?.newScore ?? trust?.new_score ?? 0) * 100),
        }),
        t("agentChat.swarm.cards.workspace.learningRecordLine", {
          count: records.length,
        }),
      ],
      state: quality?.passed ? "done" : "reworking",
    }
  }

  if (artifactCreated) {
    return {
      id: "workspace-artifact-created",
      title: t("agentChat.swarm.cards.workspace.artifactCreatedTitle"),
      meta: artifactCreated.artifactType ?? artifactCreated.artifact_type ?? "artifact",
      description: t("agentChat.swarm.cards.workspace.artifactCreatedDescription"),
      icon: FileCheck2,
      lines: [
        t("agentChat.swarm.cards.workspace.artifactLine", {
          id: artifactCreated.id.slice(0, 8),
          title: artifactCreated.title,
        }),
        artifactCreated.preview ?? artifactCreated.fileName ?? t("agentChat.swarm.cards.workspace.savedReportLine"),
      ],
      state: "done",
    }
  }

  if (completed) {
    return {
      id: "workspace-run-completed",
      title: t("agentChat.swarm.cards.workspace.completedTitle"),
      meta: readWorkspaceRunId(completed),
      description: t("agentChat.swarm.cards.workspace.completedDescription"),
      icon: CheckCircle2,
      lines: readWorkspaceRunLines(completed, t),
      state: "done",
    }
  }

  if (taskUpdate) {
    return {
      id: "workspace-task-updated",
      title: t("agentChat.swarm.cards.workspace.taskUpdatedTitle"),
      meta: taskUpdate.status,
      description: t("agentChat.swarm.cards.workspace.taskUpdatedDescription"),
      icon: FileText,
      lines: [
        t("agentChat.swarm.cards.workspace.runTaskLine", {
          id: taskUpdate.id,
          status: taskUpdate.status,
        }),
      ],
      state: taskUpdate.status === "failed" ? "failed" : "working",
    }
  }

  if (tasksLinked) {
    return {
      id: "workspace-tasks-linked",
      title: t("agentChat.swarm.cards.workspace.tasksLinkedTitle"),
      meta: t("agentChat.swarm.cards.workspace.tasksMeta", {
        count: tasksLinked.tasks.length,
      }),
      description: t("agentChat.swarm.cards.workspace.tasksLinkedDescription"),
      icon: Archive,
      lines: tasksLinked.tasks
        .slice(0, 2)
        .map((task) =>
          t("agentChat.swarm.cards.workspace.runTaskLine", {
            id: task.id,
            status: task.status,
          })
        ),
      state: "working",
    }
  }

  if (!linked) return null

  return {
    id: "workspace-run-linked",
    title: t("agentChat.swarm.cards.workspace.linkedTitle"),
    meta: linked.createdWorkspace
      ? t("agentChat.swarm.cards.workspace.createdMeta")
      : t("agentChat.swarm.cards.workspace.reusedMeta"),
    description: t("agentChat.swarm.cards.workspace.linkedDescription"),
    icon: Database,
    lines: [
      linked.workspace.title,
      t("agentChat.swarm.cards.workspace.runLine", {
        id: linked.run.id,
        status: linked.run.status,
      }),
    ],
    state: "working",
  }
}

function findLatest<TType extends AgentRuntimeEvent["type"]>(
  runtimeEvents: AgentRuntimeEvent[],
  type: TType
): Extract<AgentRuntimeEvent, { type: TType }> | null {
  for (let index = runtimeEvents.length - 1; index >= 0; index -= 1) {
    const event = runtimeEvents[index]
    if (event.type === type) {
      return event as Extract<AgentRuntimeEvent, { type: TType }>
    }
  }
  return null
}

function readWorkspaceRunLines(
  run: Extract<AgentRuntimeEvent, { type: "workspace_run_completed" | "workspace_run_failed" }>,
  t: TFunction
) {
  return [
    t("agentChat.swarm.cards.workspace.runLine", {
      id: run.id,
      status: run.status,
    }),
    run.summary ?? t("agentChat.swarm.cards.workspace.savedReportLine"),
  ]
}

function readWorkspaceRunId(run: { id: string }) {
  return run.id.slice(0, 8)
}
