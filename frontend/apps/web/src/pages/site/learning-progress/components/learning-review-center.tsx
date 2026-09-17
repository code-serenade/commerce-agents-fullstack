import { BookOpenCheck, MessageSquareText, RefreshCcw } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge, Button, Separator } from "@workspace/ui-components"
import type { LearningDomainAssessment, LearningEvidenceEntry } from "@/api"
import { buildLearningReviewItems } from "../domain/learning-support-surfaces"

export function LearningReviewCenter({
  assessments,
  evidence,
  hasError,
  isLoading,
  onContinueChat,
  onOpenConversation,
}: {
  assessments: LearningDomainAssessment[]
  evidence?: LearningEvidenceEntry[]
  hasError: boolean
  isLoading: boolean
  onContinueChat: () => void
  onOpenConversation: (conversationId: string) => void
}) {
  const { t } = useTranslation()
  const items = buildLearningReviewItems(evidence, assessments)
  const incorrectCount = items.filter(
    (item) => item.outcome === "incorrect"
  ).length
  const partialCount = items.length - incorrectCount

  return (
    <section className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <RefreshCcw aria-hidden="true" className="size-4 text-primary" />
            <h2 className="text-sm font-semibold text-foreground">
              {t("learningProgress.reviewCenter.title")}
            </h2>
          </div>
          <p className="mt-1 text-xs/5 text-muted-foreground">
            {t("learningProgress.reviewCenter.description")}
          </p>
        </div>
        <Button size="sm" onClick={onContinueChat}>
          {t("learningProgress.reviewCenter.continuePractice")}
        </Button>
      </div>

      <dl className="mt-4 grid grid-cols-3 divide-x divide-(--agent-chat-soft-border) border-y border-(--agent-chat-soft-border) py-3">
        <ReviewMetric
          label={t("learningProgress.reviewCenter.total")}
          value={items.length}
        />
        <ReviewMetric
          label={t("learningProgress.reviewCenter.incorrect")}
          value={incorrectCount}
        />
        <ReviewMetric
          label={t("learningProgress.reviewCenter.partial")}
          value={partialCount}
        />
      </dl>

      {isLoading ? (
        <ReviewState text={t("learningProgress.reviewCenter.loading")} />
      ) : hasError ? (
        <ReviewState text={t("learningProgress.reviewCenter.loadError")} />
      ) : items.length === 0 ? (
        <div className="py-12 text-center">
          <BookOpenCheck
            aria-hidden="true"
            className="mx-auto size-6 text-primary"
          />
          <p className="mt-3 text-sm font-medium text-foreground">
            {t("learningProgress.reviewCenter.emptyTitle")}
          </p>
          <p className="mx-auto mt-1 max-w-xl text-xs/5 text-muted-foreground">
            {t("learningProgress.reviewCenter.emptyDescription")}
          </p>
        </div>
      ) : (
        <div className="mt-4">
          {items.map((item, index) => (
            <article key={item.id} className="py-4 first:pt-0 last:pb-0">
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
                        `learningProgress.reviewCenter.outcomes.${item.outcome}`
                      )}
                    </span>
                    <time
                      dateTime={item.createdAt}
                      className="text-xs text-muted-foreground"
                    >
                      {new Date(item.createdAt).toLocaleString()}
                    </time>
                  </div>
                  <p className="mt-2 text-sm/6 font-medium text-foreground">
                    {item.questionText ||
                      t("learningProgress.reviewCenter.questionUnavailable")}
                  </p>
                  {item.selectedChoiceText ? (
                    <p className="mt-1 text-xs/5 text-muted-foreground">
                      {t("learningProgress.reviewCenter.selectedChoice", {
                        choice: item.selectedChoiceText,
                      })}
                    </p>
                  ) : null}
                </div>
                {item.conversationId ? (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => onOpenConversation(item.conversationId!)}
                  >
                    <MessageSquareText aria-hidden="true" className="size-4" />
                    {t("learningProgress.reviewCenter.openConversation")}
                  </Button>
                ) : null}
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  )
}

function ReviewMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="px-3 first:pl-0 last:pr-0">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="mt-1 text-lg font-semibold text-foreground">{value}</dd>
    </div>
  )
}

function ReviewState({ text }: { text: string }) {
  return <p className="py-12 text-center text-sm text-muted-foreground">{text}</p>
}
