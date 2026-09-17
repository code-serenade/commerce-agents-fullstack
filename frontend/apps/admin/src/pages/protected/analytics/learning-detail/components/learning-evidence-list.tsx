import type { TFunction } from "i18next"
import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components"
import type { LearningEvidenceSummary } from "@/api"

function getDomainLabel(t: TFunction, code: string) {
  const key = `admin.analytics.learning.detail.domains.${code}`
  const label = t(key)
  return label === key ? code : label
}

function getEvidenceDirectionLabel(t: TFunction, direction: string) {
  const key = `admin.analytics.learning.detail.evidenceDirection.${direction}`
  const label = t(key)
  return label === key ? direction : label
}

function getAssessmentOutcomeLabel(t: TFunction, outcome: string) {
  const key = `admin.analytics.learning.detail.assessmentOutcome.${outcome}`
  const label = t(key)
  return label === key ? outcome : label
}

function getEvidenceKindLabel(
  t: TFunction,
  assessmentOutcome: string | null | undefined
) {
  const kind =
    assessmentOutcome === "observed_unscored"
      ? "conversationObservation"
      : assessmentOutcome && assessmentOutcome !== "legacy_unscored"
        ? "structuredAssessment"
        : "historical"
  return t(`admin.analytics.learning.detail.evidenceKind.${kind}`)
}

function formatDateTime(value: string | null | undefined) {
  return value ? new Date(value).toLocaleString() : "-"
}

export function LearningEvidenceList({
  items,
}: {
  items: LearningEvidenceSummary[]
}) {
  const { t } = useTranslation()

  return (
    <section className="rounded-xl border border-(--app-border) bg-(--app-surface) p-4">
      <h2 className="text-sm font-semibold text-foreground">
        {t("admin.analytics.learning.detail.sections.evidence.title")}
      </h2>
      <p className="mt-1 text-xs text-muted-foreground">
        {t("admin.analytics.learning.detail.sections.evidence.description")}
      </p>
      {items.length > 0 ? (
        <div className="mt-3 space-y-2">
          {items.map((item) => {
            const hasStructuredAnswer = Boolean(
              item.questionText?.trim() && item.selectedChoiceText?.trim()
            )
            return (
              <div
                key={item.id}
                className="border border-(--app-border) px-3 py-2 text-sm"
              >
                <div className="flex items-center justify-between gap-3">
                  <div className="flex min-w-0 flex-wrap gap-1.5">
                    <Badge variant="secondary">
                      {getDomainLabel(t, item.knowledgeCode)}
                    </Badge>
                    <Badge variant="outline">
                      {getEvidenceKindLabel(t, item.assessmentOutcome)}
                    </Badge>
                    <Badge variant="outline">
                      {getAssessmentOutcomeLabel(
                        t,
                        item.assessmentOutcome ?? "legacy_unscored"
                      )}
                    </Badge>
                  </div>
                  <span className="shrink-0 text-xs text-muted-foreground">
                    {formatDateTime(item.createdAt)}
                  </span>
                </div>
                {hasStructuredAnswer ? (
                  <div className="mt-3 grid gap-3 border-l-2 border-primary pl-3">
                    <div className="min-w-0">
                      <p className="text-xs font-medium text-muted-foreground">
                        {t("admin.analytics.learning.detail.fields.question")}
                      </p>
                      <p className="mt-1 wrap-break-word text-sm/6 font-medium text-foreground">
                        {item.questionText}
                      </p>
                    </div>
                    <div className="min-w-0">
                      <p className="text-xs font-medium text-muted-foreground">
                        {t(
                          "admin.analytics.learning.detail.fields.selectedChoice"
                        )}
                      </p>
                      <p className="mt-1 wrap-break-word text-sm/6 text-foreground">
                        {item.selectedChoiceText}
                      </p>
                    </div>
                  </div>
                ) : (
                  <p className="mt-2 wrap-break-word text-sm/6 text-foreground">
                    {item.summary}
                  </p>
                )}
                <p className="mt-2 text-xs text-muted-foreground">
                  {t("admin.analytics.learning.detail.evidenceSignal", {
                    direction: getEvidenceDirectionLabel(t, item.direction),
                    strength: String(Math.round(item.strength * 100)),
                  })}
                </p>
              </div>
            )
          })}
        </div>
      ) : (
        <p className="mt-3 text-sm text-muted-foreground">
          {t("admin.analytics.learning.detail.sections.evidence.empty")}
        </p>
      )}
    </section>
  )
}
