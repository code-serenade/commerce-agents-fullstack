import { Archive, MessageSquareText } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge, Button, Separator } from "@workspace/ui-components"
import type { LearningPortfolioDay } from "../types"
import type { LearningEvidenceRange } from "@/api"
import { LearningAnswerReviewControls } from "./learning-answer-review-controls"

export function LearningPortfolio({
  days,
  hasError,
  isLoading,
  onContinueChat,
  onOpenConversation,
  summary,
  evidenceRange,
  onEvidenceRangeChange,
  reviewOnly,
  onReviewOnlyChange,
}: {
  days: LearningPortfolioDay[]
  hasError: boolean
  isLoading: boolean
  onContinueChat: () => void
  onOpenConversation: (conversationId: string) => void
  summary: { total: number; correct: number; review: number }
  evidenceRange: LearningEvidenceRange
  onEvidenceRangeChange: (range: LearningEvidenceRange) => void
  reviewOnly: boolean
  onReviewOnlyChange: (reviewOnly: boolean) => void
}) {
  const { t } = useTranslation()

  return (
    <section className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Archive aria-hidden="true" className="size-4 text-primary" />
            <h2 className="text-sm font-semibold text-foreground">
              {t("learningProgress.portfolio.title")}
            </h2>
          </div>
          <p className="mt-1 text-xs/5 text-muted-foreground">
            {t("learningProgress.portfolio.description")}
          </p>
        </div>
        <Button size="sm" onClick={onContinueChat}>
          <MessageSquareText aria-hidden="true" className="size-4" />
          {t("learningProgress.portfolio.continueChat")}
        </Button>
      </div>

      <dl className="mt-4 grid grid-cols-3 divide-x divide-(--agent-chat-soft-border) border-y border-(--agent-chat-soft-border) py-3 sm:max-w-xl">
        <PortfolioMetric
          label={t("learningProgress.portfolio.total")}
          value={summary.total}
        />
        <PortfolioMetric
          label={t("learningProgress.portfolio.correct")}
          value={summary.correct}
        />
        <PortfolioMetric
          label={t("learningProgress.portfolio.toReview")}
          value={summary.review}
        />
      </dl>

      <LearningAnswerReviewControls
        evidenceRange={evidenceRange}
        onEvidenceRangeChange={onEvidenceRangeChange}
        onReviewOnlyChange={onReviewOnlyChange}
        reviewOnly={reviewOnly}
      />
      <p className="mt-3 text-xs/5 text-muted-foreground">
        {t("learningProgress.portfolio.rangeNotice")}
      </p>

      {isLoading ? (
        <PortfolioState text={t("learningProgress.portfolio.loading")} />
      ) : hasError ? (
        <PortfolioState text={t("learningProgress.portfolio.loadError")} />
      ) : days.length === 0 ? (
        <div className="py-12 text-center">
          <Archive aria-hidden="true" className="mx-auto size-6 text-primary" />
          <p className="mt-3 text-sm font-medium text-foreground">
            {t(reviewOnly
              ? "learningProgress.reviewCenter.emptyTitle"
              : "learningProgress.portfolio.emptyTitle")}
          </p>
          <p className="mx-auto mt-1 max-w-xl text-xs/5 text-muted-foreground">
            {t(reviewOnly
              ? "learningProgress.reviewCenter.emptyDescription"
              : "learningProgress.portfolio.emptyDescription")}
          </p>
        </div>
      ) : (
        <div className="mt-5 space-y-6">
          {days.map((day) => (
            <section key={day.date} aria-labelledby={`portfolio-${day.date}`}>
              <div className="flex items-center justify-between gap-3 border-b border-(--agent-chat-soft-border) pb-2">
                <h3
                  id={`portfolio-${day.date}`}
                  className="text-xs font-semibold text-foreground"
                >
                  {new Date(`${day.date}T00:00:00`).toLocaleDateString()}
                </h3>
                <span className="text-[11px] text-muted-foreground">
                  {t("learningProgress.portfolio.dayCount", {
                    count: day.items.length,
                  })}
                </span>
              </div>
              <div>
                {day.items.map((item, index) => (
                  <article key={item.id} className="py-4 last:pb-0">
                    {index > 0 ? (
                      <div className="mb-4">
                        <Separator />
                      </div>
                    ) : null}
                    <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          <Badge variant="secondary">
                            {t(`learningProgress.domains.${item.knowledgeCode}`)}
                          </Badge>
                          <span className="text-xs font-semibold text-primary">
                            {t(
                              `learningProgress.portfolio.outcomes.${item.outcome}`
                            )}
                          </span>
                          <time
                            dateTime={item.createdAt}
                            className="text-xs text-muted-foreground"
                          >
                            {new Date(item.createdAt).toLocaleTimeString()}
                          </time>
                        </div>
                        <p className="mt-2 text-sm/6 font-medium text-foreground">
                          {item.questionText ||
                            t("learningProgress.portfolio.questionUnavailable")}
                        </p>
                        {item.selectedChoiceText ? (
                          <p className="mt-1 text-xs/5 text-muted-foreground">
                            {t("learningProgress.portfolio.selectedChoice", {
                              choice: item.selectedChoiceText,
                            })}
                          </p>
                        ) : null}
                      </div>
                      {item.conversationId ? (
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() =>
                            onOpenConversation(item.conversationId!)
                          }
                        >
                          <MessageSquareText
                            aria-hidden="true"
                            className="size-4"
                          />
                          {t("learningProgress.portfolio.openConversation")}
                        </Button>
                      ) : null}
                    </div>
                  </article>
                ))}
              </div>
            </section>
          ))}
        </div>
      )}
    </section>
  )
}

function PortfolioMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="px-3 first:pl-0 last:pr-0">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="mt-1 text-lg font-semibold text-foreground">{value}</dd>
    </div>
  )
}

function PortfolioState({ text }: { text: string }) {
  return <p className="py-12 text-center text-sm text-muted-foreground">{text}</p>
}
