import type { TFunction } from "i18next"
import type {
  AgentDispatchFact,
  AgentProcessProgress,
} from "./agent-process-kind-view-model"
import type {
  DispatchWorkbenchStep,
} from "./agent-dispatch-workbench-view-model"
import { isInternalRuntimeText } from "./agent-user-facing-runtime-text"

type PreviewFallbackKey =
  | "fallbackFactDetail"
  | "fallbackFactLabel"
  | "fallbackMode"
  | "fallbackStage"
  | "fallbackStepDetail"
  | "fallbackStepTitle"
  | "fallbackSummary"

const fallbackCopy: Record<PreviewFallbackKey, string> = {
  fallbackFactDetail: "Task material status synced",
  fallbackFactLabel: "Task materials",
  fallbackMode: "Task process",
  fallbackStage: "Working on task",
  fallbackStepDetail: "Step status syncing",
  fallbackStepTitle: "Task step",
  fallbackSummary: "Preparing the answer for this task",
}

export function buildDispatchPreviewLines(input: {
  adaptiveSummary?: string
  facts: AgentDispatchFact[]
  modeLabel: string
  progress: AgentProcessProgress
  request: string
  steps: DispatchWorkbenchStep[]
  summary: string
  t: TFunction
}) {
  const { adaptiveSummary, facts, modeLabel, progress, request, steps, summary, t } = input
  const readableMode = sanitizePreviewText(
    modeLabel,
    fallbackTranslation(t, "fallbackMode")
  )
  const readableSummary = sanitizePreviewText(
    summary,
    fallbackTranslation(t, "fallbackSummary")
  )

  return [
    t("agentChat.thread.dispatchPanel.preview.heading", { mode: readableMode }),
    t("agentChat.thread.dispatchPanel.preview.request", { request }),
    t("agentChat.thread.dispatchPanel.preview.summary", {
      summary: readableSummary,
    }),
    ...(adaptiveSummary
      ? [
          t("agentChat.thread.dispatchPanel.preview.adaptive", {
            summary: sanitizePreviewText(
              adaptiveSummary,
              fallbackTranslation(t, "fallbackSummary")
            ),
          }),
        ]
      : []),
    t("agentChat.thread.dispatchPanel.preview.progress", {
      completed: progress.completed,
      stage: sanitizePreviewText(
        progress.stage,
        fallbackTranslation(t, "fallbackStage")
      ),
      total: progress.total,
    }),
    ...steps.map((step) => {
      const readableStep = sanitizePreviewStep(step, t)
      return t("agentChat.thread.dispatchPanel.preview.step", {
        detail: readableStep.detail,
        index: String(step.index + 1).padStart(2, "0"),
        title: readableStep.title,
      })
    }),
    ...facts.map((fact) => {
      const readableFact = sanitizePreviewFact(fact, t)
      return t("agentChat.thread.dispatchPanel.preview.fact", {
        detail: readableFact.detail,
        label: readableFact.label,
      })
    }),
  ]
}

function sanitizePreviewStep(step: DispatchWorkbenchStep, t: TFunction) {
  return {
    detail: sanitizePreviewText(
      step.detail,
      fallbackTranslation(t, "fallbackStepDetail")
    ),
    title: sanitizePreviewText(
      step.title,
      fallbackTranslation(t, "fallbackStepTitle")
    ),
  }
}

function sanitizePreviewFact(fact: AgentDispatchFact, t: TFunction) {
  return {
    detail: sanitizePreviewText(
      fact.detail,
      fallbackTranslation(t, "fallbackFactDetail")
    ),
    label: sanitizePreviewText(
      fact.label,
      fallbackTranslation(t, "fallbackFactLabel")
    ),
  }
}

function fallbackTranslation(t: TFunction, key: PreviewFallbackKey) {
  const value = t(`agentChat.thread.dispatchPanel.preview.${key}`)
  return isI18nKeyLeak(value) ? fallbackCopy[key] : value
}

function sanitizePreviewText(value: string, fallback: string) {
  const trimmed = value.trim()
  if (!trimmed || isI18nKeyLeak(trimmed) || isInternalRuntimeText(trimmed)) {
    return fallback
  }
  return trimmed
}

function isI18nKeyLeak(value: string) {
  return /^agentChat\./.test(value)
}
