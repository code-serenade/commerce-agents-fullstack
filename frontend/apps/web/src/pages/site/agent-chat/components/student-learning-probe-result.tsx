import {
  BookOpenCheck,
  CheckCircle2,
  Clock3,
  ListPlus,
  Loader2,
  RefreshCw,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type { LearningProbeResultResponse } from "@/api"
import { StudentLearningProbeExplanation } from "./student-learning-probe-explanation"
import { StudentLearningProbeOutcome } from "./student-learning-probe-outcome"

export function StudentLearningProbeResult({
  continuing = false,
  continueFailed = false,
  continueUnavailable = false,
  evaluationPending = false,
  resultFailed = false,
  onContinue,
  onRetryResult,
  onViewProgress,
  result,
}: {
  continuing?: boolean
  continueFailed?: boolean
  continueUnavailable?: boolean
  evaluationPending?: boolean
  resultFailed?: boolean
  onDismiss?: () => void
  onContinue?: () => void
  onRetryResult?: () => void
  onViewProgress?: () => void
  result: LearningProbeResultResponse | null
}) {
  const { t } = useTranslation()
  const outcome = result?.outcome

  if (!outcome) {
    if (resultFailed) {
      return (
        <div
          data-learning-probe-result="true"
          className="mt-3 flex flex-wrap items-center gap-2 rounded-md border border-destructive/35 bg-destructive/5 px-3 py-2 text-[11px]/5 text-foreground"
          role="alert"
        >
          <Clock3 className="size-3.5 shrink-0 text-destructive" />
          <span className="min-w-0 flex-1">
            {t("agentChat.thread.learningProbe.resultFailed")}
          </span>
          {onRetryResult ? (
            <button
              type="button"
              className="inline-flex h-7 shrink-0 items-center gap-1 rounded-md px-2 font-medium text-primary transition-colors hover:bg-muted"
              onClick={onRetryResult}
            >
              <RefreshCw className="size-3" />
              {t("agentChat.thread.learningProbe.retryResult")}
            </button>
          ) : null}
        </div>
      )
    }
    return (
      <div
        data-learning-probe-result="true"
        aria-atomic="true"
        aria-live="polite"
        className="mt-3 flex items-start gap-2 rounded-md border border-(--agent-chat-soft-border) bg-(--agent-chat-accent-soft) px-3 py-2 text-[11px]/5 text-muted-foreground"
        role="status"
      >
        <Clock3 className="mt-0.5 size-3.5 shrink-0 text-primary" />
        <span>
          {evaluationPending || result?.status === "answered"
            ? t("agentChat.thread.learningProbe.evaluating")
            : t("agentChat.thread.learningProbe.waitingAnswer")}
        </span>
      </div>
    )
  }

  return (
    <div
      data-learning-probe-result="true"
      aria-atomic="true"
      aria-live="polite"
      className="mt-3 rounded-md border border-(--agent-chat-soft-border) bg-(--agent-chat-accent-soft) p-3"
      role="status"
    >
      <div className="flex items-center gap-2 text-xs font-semibold text-foreground">
        <CheckCircle2 className="size-3.5 shrink-0 text-primary" />
        <span className="min-w-0 flex-1">
          {t("agentChat.thread.learningProbe.resultTitle")}
        </span>
      </div>
      <p className="mt-1 text-xs/5">
        <StudentLearningProbeOutcome outcome={outcome} />
      </p>
      <StudentLearningProbeExplanation outcome={outcome} result={result} />
      <div className="mt-3 flex flex-wrap items-center justify-between gap-2 border-t border-(--agent-chat-soft-border) pt-3">
        <span className="inline-flex items-center gap-1.5 text-[11px] font-medium text-foreground">
          <BookOpenCheck className="size-3.5 shrink-0 text-primary" />
          {t("agentChat.thread.learningProbe.recorded")}
        </span>
        {onViewProgress ? (
          <button
            type="button"
            className="text-[11px] font-medium text-primary hover:underline"
            onClick={onViewProgress}
          >
            {t("agentChat.thread.learningProbe.viewRecord")}
          </button>
        ) : null}
      </div>
      {result?.canContinue && onContinue ? (
        <div className="mt-3 border-t border-(--agent-chat-soft-border) pt-3">
          {continueFailed || continueUnavailable ? (
            <div
              className="mb-2 flex items-start gap-2 text-[11px]/5 text-muted-foreground"
              role={continueFailed ? "alert" : "status"}
            >
              <Clock3 className="mt-0.5 size-3.5 shrink-0 text-primary" />
              <span>
                {continueFailed
                  ? t("agentChat.thread.learningProbe.continueFailed")
                  : t("agentChat.thread.learningProbe.continueUnavailable")}
              </span>
            </div>
          ) : null}
          {!continueUnavailable ? (
            <button
              type="button"
              disabled={continuing}
              onClick={onContinue}
              className="inline-flex h-8 items-center gap-1.5 rounded-md bg-(--agent-chat-elevated-surface) px-3 text-[11px] font-medium text-primary transition-colors hover:bg-muted disabled:cursor-wait disabled:opacity-60"
            >
              {continuing ? (
                <Loader2 className="size-3 animate-spin" />
              ) : (
                <ListPlus className="size-3" />
              )}
              {continuing
                ? t("agentChat.thread.learningProbe.continuing")
                : continueFailed
                  ? t("agentChat.thread.learningProbe.retryContinue")
                  : t("agentChat.thread.learningProbe.continue")}
            </button>
          ) : null}
        </div>
      ) : result?.canContinue === false && onContinue ? (
        <div className="mt-3 flex items-start gap-2 border-t border-(--agent-chat-soft-border) pt-3 text-[11px]/5 text-muted-foreground">
          <CheckCircle2 className="mt-0.5 size-3.5 shrink-0 text-primary" />
          <span>{t("agentChat.thread.learningProbe.continuationComplete")}</span>
        </div>
      ) : null}
    </div>
  )
}
