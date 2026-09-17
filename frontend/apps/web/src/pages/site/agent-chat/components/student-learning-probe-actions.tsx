import { Check, ChevronUp, Loader2, Send } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Alert } from "@workspace/ui-components/stable/alert"

export function StudentLearningProbeActions({
  choiceSubmitFailed,
  engageFailed,
  isSingleChoice,
  selectedChoiceId,
  starting,
  submittingChoice,
  onStartAnswer,
  onSubmitChoice,
  onCollapse,
  onViewProgress,
}: {
  choiceSubmitFailed: boolean
  engageFailed: boolean
  isSingleChoice: boolean
  selectedChoiceId: string | null
  starting: boolean
  submittingChoice: boolean
  onStartAnswer: () => void
  onSubmitChoice: () => void
  onCollapse: () => void
  onViewProgress: () => void
}) {
  const { t } = useTranslation()

  return (
    <>
      {choiceSubmitFailed ? (
        <div className="mb-3" role="alert">
          <Alert
            variant="error"
            description={t("agentChat.thread.learningProbe.submitChoiceFailed")}
          />
        </div>
      ) : null}
      {engageFailed ? (
        <div className="mb-3" role="alert">
          <Alert
            variant="error"
            description={t("agentChat.thread.learningProbe.engageFailed")}
          />
        </div>
      ) : null}
      <div className="flex flex-wrap gap-2">
        {isSingleChoice ? (
          <button
            type="button"
            disabled={!selectedChoiceId || submittingChoice}
            onClick={onSubmitChoice}
            className="inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-[11px] font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {submittingChoice ? (
              <Loader2 className="size-3 animate-spin" />
            ) : (
              <Check className="size-3" />
            )}
            {submittingChoice
              ? t("agentChat.thread.learningProbe.submittingChoice")
              : t("agentChat.thread.learningProbe.submitChoice")}
          </button>
        ) : null}
        {!isSingleChoice ? (
          <button
            type="button"
            disabled={starting}
            onClick={onStartAnswer}
            className="inline-flex h-8 items-center gap-1.5 rounded-md bg-(--agent-chat-accent-soft) px-3 text-[11px] font-medium text-foreground transition-colors hover:text-primary disabled:cursor-wait disabled:opacity-60"
          >
            <Send className="size-3" />
            {t("agentChat.thread.learningProbe.answer")}
          </button>
        ) : null}
        <button
          type="button"
          disabled={starting || submittingChoice}
          onClick={onCollapse}
          className="inline-flex h-8 items-center gap-1.5 rounded-md border border-border bg-background px-3 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-muted disabled:cursor-wait disabled:opacity-60"
        >
          <ChevronUp className="size-3" />
          {t("agentChat.thread.learningProbe.later")}
        </button>
        {!isSingleChoice ? (
          <button
            type="button"
            disabled={starting}
            onClick={onViewProgress}
            className="inline-flex h-8 items-center gap-1.5 rounded-md px-3 text-[11px] font-medium text-primary transition-colors hover:bg-muted disabled:cursor-wait disabled:opacity-60"
          >
            {t("agentChat.thread.learningProbe.viewProgress")}
          </button>
        ) : null}
      </div>
    </>
  )
}
