import type { TFunction } from "i18next"
import { useTranslation } from "react-i18next"
import { MessageSquareText } from "lucide-react"
import { Badge, Button, Separator } from "@workspace/ui-components"
import type { LearningEvidenceEntry } from "@/api"

export function LearningEvidenceList({
  evidence,
  hasError,
  isLoading,
  onOpenConversation,
}: {
  evidence?: LearningEvidenceEntry[]
  hasError: boolean
  isLoading: boolean
  onOpenConversation: (conversationId: string) => void
}) {
  const { t } = useTranslation()

  if (isLoading) {
    return (
      <p className="mt-2 text-sm/6 text-muted-foreground">
        {t("learningProgress.evidenceLoading")}
      </p>
    )
  }

  if (hasError) {
    return (
      <p className="mt-2 text-sm/6 text-muted-foreground">
        {t("learningProgress.evidenceLoadError")}
      </p>
    )
  }

  if (!evidence || evidence.length === 0) {
    return (
      <p className="mt-2 text-sm/6 text-muted-foreground">
        {t("learningProgress.evidenceEmpty")}
      </p>
    )
  }

  return (
    <div className="mt-3 space-y-5">
      {groupEvidenceByDate(t, evidence).map((group) => (
        <section key={group.key} aria-label={group.label}>
          <div className="flex items-center gap-3">
            <p className="shrink-0 text-xs font-medium text-muted-foreground">
              {group.label}
            </p>
            <div className="flex-1">
              <Separator />
            </div>
          </div>
          <div className="mt-2 border-l border-(--agent-chat-soft-border) pl-4">
            {group.items.map((item, index) => (
              <article
                key={item.id}
                className={index === 0 ? "relative" : "relative mt-3"}
              >
                <span
                  className="absolute top-3 -left-[21px] size-2 rounded-full border-2 border-(--agent-chat-elevated-surface) bg-primary"
                  aria-hidden="true"
                />
                <div className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) px-3 py-2.5 shadow-sm backdrop-blur-xl">
                  <div className="flex items-center justify-between gap-3">
                    <div className="flex min-w-0 flex-wrap gap-1.5">
                      <Badge variant="secondary">
                        {getDomainLabel(t, item.knowledgeCode)}
                      </Badge>
                      <Badge variant="outline">
                        {getEvidenceTypeLabel(t, item)}
                      </Badge>
                      <Badge variant="outline">
                        {getAssessmentOutcomeLabel(
                          t,
                          item.assessmentOutcome ?? "legacy_unscored"
                        )}
                      </Badge>
                    </div>
                    <time
                      dateTime={item.createdAt}
                      className="shrink-0 text-xs text-muted-foreground"
                    >
                      {formatTime(item.createdAt)}
                    </time>
                  </div>
                  {item.questionText && item.selectedChoiceText ? (
                    <dl className="mt-2 grid gap-1.5 text-sm/5">
                      <div className="flex min-w-0 items-start gap-2">
                        <dt className="shrink-0 text-[11px] font-medium text-primary">
                          {t("learningProgress.evidenceQuestion")}
                        </dt>
                        <dd className="min-w-0 text-foreground">
                          {item.questionText}
                        </dd>
                      </div>
                      <div className="flex min-w-0 items-start gap-2">
                        <dt className="shrink-0 text-[11px] font-medium text-muted-foreground">
                          {t("learningProgress.evidenceSelectedChoice")}
                        </dt>
                        <dd className="min-w-0 font-medium text-foreground">
                          {item.selectedChoiceText}
                        </dd>
                      </div>
                    </dl>
                  ) : (
                    <p className="mt-1 text-sm/5 text-foreground">
                      {truncateSummary(item.summary, 120)}
                    </p>
                  )}
                  <div className="mt-1 flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
                    <p className="text-xs text-muted-foreground">
                      {t(
                        item.assessmentOutcome &&
                          isScoredOutcome(item.assessmentOutcome)
                          ? "learningProgress.assessmentScore"
                          : "learningProgress.evidenceSignal",
                        {
                          strength: String(Math.round(item.strength * 100)),
                        }
                      )}
                    </p>
                    {item.conversationId ? (
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        className="h-7 px-2 text-xs"
                        onClick={() => {
                          if (item.conversationId) {
                            onOpenConversation(item.conversationId)
                          }
                        }}
                      >
                        <MessageSquareText className="size-3.5" />
                        {t("learningProgress.viewRelatedConversation")}
                      </Button>
                    ) : null}
                  </div>
                </div>
              </article>
            ))}
          </div>
        </section>
      ))}
    </div>
  )
}

function groupEvidenceByDate(t: TFunction, evidence: LearningEvidenceEntry[]) {
  const groups = new Map<
    string,
    { key: string; label: string; items: LearningEvidenceEntry[] }
  >()

  evidence.forEach((item) => {
    const date = new Date(item.createdAt)
    const key = getLocalDateKey(date)
    const group = groups.get(key)

    if (group) {
      group.items.push(item)
      return
    }

    groups.set(key, {
      key,
      label: formatDateLabel(t, date),
      items: [item],
    })
  })

  return [...groups.values()]
}

function getLocalDateKey(date: Date) {
  return [date.getFullYear(), date.getMonth() + 1, date.getDate()]
    .map((value) => String(value).padStart(2, "0"))
    .join("-")
}

function formatDateLabel(t: TFunction, date: Date) {
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(today.getDate() - 1)

  if (getLocalDateKey(date) === getLocalDateKey(today)) {
    return t("learningProgress.timelineToday")
  }

  if (getLocalDateKey(date) === getLocalDateKey(yesterday)) {
    return t("learningProgress.timelineYesterday")
  }

  return date.toLocaleDateString()
}

function formatTime(value: string) {
  return new Date(value).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  })
}

function getDomainLabel(t: TFunction, code: string) {
  const key = `learningProgress.domains.${code}`
  const label = t(key)
  return label === key ? code : label
}

function getEvidenceTypeLabel(t: TFunction, item: LearningEvidenceEntry) {
  const typeKey =
    item.evidenceType === "probe_answer" ? "probeAnswer" : item.evidenceType
  const label = t(`learningProgress.evidenceTypes.${typeKey}`)
  return label === `learningProgress.evidenceTypes.${typeKey}`
    ? item.evidenceType
    : label
}

function getAssessmentOutcomeLabel(t: TFunction, outcome: string) {
  const key = `learningProgress.assessmentOutcomes.${outcome}`
  const label = t(key)
  return label === key ? outcome : label
}

function isScoredOutcome(outcome: string) {
  return (
    outcome === "correct" || outcome === "partial" || outcome === "incorrect"
  )
}

function truncateSummary(value: string, max: number) {
  const chars = Array.from(value)
  if (chars.length <= max) return value
  return `${chars.slice(0, max).join("")}…`
}
