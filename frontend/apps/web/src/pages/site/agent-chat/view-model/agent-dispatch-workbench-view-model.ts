import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  formatRuntimeRole,
  getAgentProcessKind,
  getAgentProcessProgress,
  getAgentProcessSummary,
  getAgentProcessTitle,
  getDispatchFacts,
  getProcessStepRows,
  type AgentDispatchFact,
  type AgentProcessKind,
  type AgentProcessProgress,
} from "./agent-process-kind-view-model"
import {
  hasFinished,
  normalizeRuntimeRole,
} from "./agent-process-runtime-view-model"
import type { SwarmState } from "./swarm-panel-view-model"

export interface DispatchWorkbenchStep {
  detail: string
  id: string
  index: number
  state: SwarmState
  title: string
}

export interface DispatchWorkbenchModel {
  activeStep: DispatchWorkbenchStep
  activeStepIndex: number
  facts: AgentDispatchFact[]
  filename: string
  kind: AgentProcessKind
  modeLabel: string
  previewLines: string[]
  progress: AgentProcessProgress
  request: string
  steps: DispatchWorkbenchStep[]
  summary: string
}

export function getDispatchWorkbenchModel(
  input: {
    isStreaming: boolean
    runtimeEvents: AgentRuntimeEvent[]
    task?: string
  },
  t: TFunction
): DispatchWorkbenchModel {
  const { isStreaming, runtimeEvents, task } = input
  const kind = getAgentProcessKind(runtimeEvents)
  const rows = getProcessStepRows(runtimeEvents, isStreaming, t)
  const facts = getDispatchFacts(runtimeEvents, t)
  const total = Math.max(rows.length, 1)
  const rawProgress = getAgentProcessProgress(
    {
      completedCount: resolveCompletedCount(runtimeEvents, isStreaming, total),
      isStreaming,
      runtimeEvents,
      totalCount: total,
    },
    t
  )
  const progress = normalizeProgress(rawProgress, runtimeEvents, isStreaming, total)
  const steps: DispatchWorkbenchStep[] = rows.length
    ? rows.map(([title, detail], index) => {
        const displayTitle = formatStepTitle(title, t)
        return {
          detail: formatStepDetail(detail, t),
          id: `${index}-${displayTitle}`,
          index,
          state: resolveStepState(index, progress.completed, total, runtimeEvents, isStreaming),
          title: displayTitle,
        }
      })
    : [
        {
          detail: t("agentChat.thread.dispatchPanel.waitingStepDetail"),
          id: "waiting",
          index: 0,
          state: isStreaming ? "working" : "queued",
          title: t("agentChat.thread.dispatchPanel.waitingStepTitle"),
        },
      ]
  const activeStepIndex = resolveActiveStepIndex(steps, progress.completed)
  const activeStep = steps[activeStepIndex] ?? steps[0]!
  const request = task?.trim() || t("agentChat.thread.dispatchPanel.emptyTask")
  const summary = getAgentProcessSummary(runtimeEvents, request, t)
  const modeLabel = getAgentProcessTitle(runtimeEvents, t)

  return {
    activeStep,
    activeStepIndex,
    facts,
    filename: t(`agentChat.thread.dispatchPanel.previewFiles.${kind}`),
    kind,
    modeLabel,
    previewLines: buildPreviewLines({
      facts,
      modeLabel,
      progress,
      request,
      steps,
      summary,
      t,
    }),
    progress,
    request,
    steps,
    summary,
  }
}

function formatStepTitle(title: string, t: TFunction) {
  const normalized = normalizeRuntimeRole(title)
  if (KNOWN_ROLE_IDS.has(normalized) && isTechnicalRoleLabel(title, normalized)) {
    return formatRuntimeRole(normalized, t)
  }
  return title
}

function formatStepDetail(detail: string, t: TFunction) {
  if (isInternalOrOverlongStepDetail(detail)) {
    return t("agentChat.thread.dispatchPanel.compactedStepDetail")
  }
  return formatKnownRuntimeTerms(detail, t)
}

function isInternalOrOverlongStepDetail(detail: string) {
  const normalized = detail.toLowerCase()
  return (
    detail.length > 180 ||
    normalized.includes("assistant:") ||
    normalized.includes("chat_worker") ||
    normalized.includes("final_synthesis") ||
    normalized.includes("system prompt") ||
    normalized.includes("user:")
  )
}

function formatKnownRuntimeTerms(detail: string, t: TFunction) {
  return detail
    .replaceAll("chat_bypass", t("agentChat.thread.dispatchPanel.routeModes.chat_bypass"))
    .replaceAll("task_graph", t("agentChat.thread.dispatchPanel.routeModes.task_graph"))
    .replaceAll("single_agent", t("agentChat.thread.dispatchPanel.routeModes.single_agent"))
}

const KNOWN_ROLE_IDS = new Set([
  "accounting",
  "chat",
  "creative",
  "data",
  "design",
  "douyin_ads",
  "engineering",
  "search",
  "ops",
  "service",
  "web",
])

function isTechnicalRoleLabel(title: string, normalized: string) {
  const normalizedTitle = title.trim().toLowerCase()
  return (
    normalizedTitle === normalized ||
    normalizedTitle === normalized.replaceAll("_", "-") ||
    normalizedTitle === normalized.replaceAll("_", " ") ||
    normalizedTitle.endsWith(`.${normalized}`) ||
    normalizedTitle.endsWith(`_${normalized}`) ||
    normalizedTitle.endsWith(`-${normalized}`)
  )
}

function normalizeProgress(
  progress: AgentProcessProgress,
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  total: number
): AgentProcessProgress {
  const completed = resolveCompletedCount(runtimeEvents, isStreaming, total, progress.completed)
  return {
    ...progress,
    completed,
    percent: Math.min(100, Math.round((completed / total) * 100)),
    total,
  }
}

function resolveCompletedCount(
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean,
  total: number,
  rawCompleted = 0
) {
  if (total <= 0) return 0
  if (hasFailed(runtimeEvents)) {
    return Math.max(1, Math.min(total, rawCompleted || 1))
  }
  if (hasFinished(runtimeEvents)) return total
  if (isStreaming) {
    return Math.max(1, Math.min(total, rawCompleted || 1))
  }
  if (runtimeEvents.length > 0) {
    return Math.max(1, Math.min(total, rawCompleted || total))
  }
  return Math.max(0, Math.min(total, rawCompleted))
}

function resolveStepState(
  index: number,
  completed: number,
  total: number,
  runtimeEvents: AgentRuntimeEvent[],
  isStreaming: boolean
): SwarmState {
  if (hasFailed(runtimeEvents) && index === Math.max(completed - 1, 0)) {
    return "failed"
  }
  if (completed >= total) return "done"
  if (index < Math.max(completed - 1, 0)) return "done"
  if (index === Math.max(completed - 1, 0)) {
    return isStreaming ? "working" : "done"
  }
  return "queued"
}

function resolveActiveStepIndex(steps: DispatchWorkbenchStep[], completed: number) {
  if (!steps.length) return 0
  const failedIndex = steps.findIndex((step) => step.state === "failed")
  if (failedIndex >= 0) return failedIndex
  const workingIndex = steps.findIndex((step) => step.state === "working")
  if (workingIndex >= 0) return workingIndex
  return Math.max(0, Math.min(steps.length - 1, completed - 1))
}

function hasFailed(runtimeEvents: AgentRuntimeEvent[]) {
  return runtimeEvents.some(
    (event) =>
      event.type === "failed" ||
      event.type === "workspace_run_failed" ||
      event.type === "task_node_failed"
  )
}

function buildPreviewLines(input: {
  facts: AgentDispatchFact[]
  modeLabel: string
  progress: AgentProcessProgress
  request: string
  steps: DispatchWorkbenchStep[]
  summary: string
  t: TFunction
}) {
  const { facts, modeLabel, progress, request, steps, summary, t } = input
  return [
    t("agentChat.thread.dispatchPanel.preview.heading", { mode: modeLabel }),
    t("agentChat.thread.dispatchPanel.preview.request", { request }),
    t("agentChat.thread.dispatchPanel.preview.summary", { summary }),
    t("agentChat.thread.dispatchPanel.preview.progress", {
      completed: progress.completed,
      stage: progress.stage,
      total: progress.total,
    }),
    ...steps.map((step) =>
      t("agentChat.thread.dispatchPanel.preview.step", {
        detail: step.detail,
        index: String(step.index + 1).padStart(2, "0"),
        title: step.title,
      })
    ),
    ...facts.map((fact) =>
      t("agentChat.thread.dispatchPanel.preview.fact", {
        detail: fact.detail,
        label: fact.label,
      })
    ),
  ]
}
