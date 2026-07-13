import { CheckCircle2, Loader2, MessageSquareText, Star } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge, Button, Textarea } from "@workspace/ui-components"
import type { AgentRunFeedbackState } from "../view-model/use-agent-run-feedback"

const RATINGS = [1, 2, 3, 4, 5] as const

export function AgentRunFeedbackPanel({
  feedback,
  compact = false,
}: {
  feedback: AgentRunFeedbackState
  compact?: boolean
}) {
  const { t } = useTranslation()
  const targetLabel = feedback.target
    ? (feedback.target.workspaceTitle ?? t("agentChat.feedback.linked"))
    : t("agentChat.feedback.noTarget")

  if (compact) {
    return (
      <div className="rounded-lg bg-(--agent-chat-elevated-strong)/70 p-2.5">
        <div className="flex min-w-0 items-center justify-between gap-2">
          <div className="flex min-w-0 items-center gap-1.5 text-xs font-medium">
            <MessageSquareText className="size-4 shrink-0 text-muted-foreground" />
            <span className="truncate">{t("agentChat.feedback.title")}</span>
          </div>
          <Badge variant={feedback.submitted ? "default" : "outline"}>
            {feedback.submitted
              ? t("agentChat.feedback.submitted")
              : t("agentChat.feedback.pending")}
          </Badge>
        </div>
        <div
          className="mt-2 flex items-center gap-1"
          aria-label={t("agentChat.feedback.ratingLabel")}
        >
          {RATINGS.map((rating) => {
            const active = rating <= feedback.rating
            return (
              <button
                key={rating}
                type="button"
                disabled={feedback.isSubmitting || feedback.submitted}
                onClick={() => feedback.setRating(rating)}
                className={[
                  "flex size-6 items-center justify-center rounded-md border transition-colors disabled:cursor-not-allowed disabled:opacity-50",
                  active
                    ? "border-amber-300 bg-amber-50 text-amber-600 dark:border-amber-500/40 dark:bg-amber-500/10 dark:text-amber-300"
                    : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground",
                ].join(" ")}
                title={t("agentChat.feedback.ratingTitle", { rating })}
                aria-label={t("agentChat.feedback.ratingTitle", { rating })}
              >
                <Star className={active ? "size-3 fill-current" : "size-3"} />
              </button>
            )
          })}
        </div>
        <p className="mt-2 truncate text-[11px] text-muted-foreground">
          {t("agentChat.feedback.target", { target: targetLabel })}
        </p>
      </div>
    )
  }

  return (
    <div className="rounded-xl border border-border bg-background p-3 shadow-sm">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex min-w-0 items-center gap-2 text-sm font-medium">
            <MessageSquareText className="size-4 shrink-0 text-muted-foreground" />
            <span className="truncate">{t("agentChat.feedback.title")}</span>
          </div>
          <p className="mt-1 line-clamp-2 text-xs/5 text-muted-foreground">
            {t("agentChat.feedback.description")}
          </p>
        </div>
        <Badge variant={feedback.submitted ? "default" : "outline"}>
          {feedback.submitted
            ? t("agentChat.feedback.submitted")
            : t("agentChat.feedback.pending")}
        </Badge>
      </div>

      <div className="mt-3 flex flex-wrap items-center gap-2">
        <span className="text-xs text-muted-foreground">
          {t("agentChat.feedback.target", { target: targetLabel })}
        </span>
        {feedback.target ? (
          <span className="rounded-full bg-muted px-2 py-1 text-[11px] text-muted-foreground">
            {t("agentChat.feedback.linked")}
          </span>
        ) : null}
      </div>

      <div
        className="mt-3 flex items-center gap-1.5"
        aria-label={t("agentChat.feedback.ratingLabel")}
      >
        {RATINGS.map((rating) => {
          const active = rating <= feedback.rating
          return (
            <button
              key={rating}
              type="button"
              disabled={feedback.isSubmitting || feedback.submitted}
              onClick={() => feedback.setRating(rating)}
              className={[
                "flex size-8 items-center justify-center rounded-md border transition-colors disabled:cursor-not-allowed disabled:opacity-50",
                active
                  ? "border-amber-300 bg-amber-50 text-amber-600 dark:border-amber-500/40 dark:bg-amber-500/10 dark:text-amber-300"
                  : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground",
              ].join(" ")}
              title={t("agentChat.feedback.ratingTitle", { rating })}
              aria-label={t("agentChat.feedback.ratingTitle", { rating })}
            >
              <Star className={active ? "size-4 fill-current" : "size-4"} />
            </button>
          )
        })}
      </div>

      <div className="mt-3">
        <Textarea
          value={feedback.comment}
          onValueChange={feedback.setComment}
          placeholder={t("agentChat.feedback.placeholder")}
          disabled={feedback.isSubmitting || feedback.submitted}
          rows={3}
        />
      </div>

      {feedback.error ? (
        <p className="mt-2 text-xs text-destructive">{feedback.error}</p>
      ) : feedback.disabledReason ? (
        <p className="mt-2 text-xs text-muted-foreground">
          {feedback.disabledReason}
        </p>
      ) : null}

      <div className="mt-3 flex justify-end">
        <Button
          size="sm"
          disabled={!feedback.canSubmit}
          onClick={() => void feedback.submit()}
        >
          {feedback.isSubmitting ? (
            <Loader2 className="size-4 animate-spin" />
          ) : feedback.submitted ? (
            <CheckCircle2 className="size-4" />
          ) : (
            <MessageSquareText className="size-4" />
          )}
          {feedback.submitted
            ? t("agentChat.feedback.submitted")
            : t("agentChat.feedback.submit")}
        </Button>
      </div>
    </div>
  )
}
