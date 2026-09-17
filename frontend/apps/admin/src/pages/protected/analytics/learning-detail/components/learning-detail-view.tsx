import {
  BadgeCheck,
  Clock3,
  ArrowLeft,
  ClipboardCheck,
  History,
  MessageSquareText,
  TriangleAlert,
} from "lucide-react"
import type { TFunction } from "i18next"
import { useTranslation } from "react-i18next"
import { MetricCards, PageLoading } from "@workspace/app-kit"
import type { MetricCardsItem } from "@workspace/app-kit"
import type { LearningStudentDetailResponse } from "@/api"
import { normalizeLearningAssessmentSummary } from "../../learning-assessment"
import { LearningEvidenceList } from "./learning-evidence-list"

function getDomainLabel(t: TFunction, code: string) {
  const key = `admin.analytics.learning.detail.domains.${code}`
  const label = t(key)
  return label === key ? code : label
}

function getDomainEvidenceStatusLabel(t: TFunction, status: string) {
  const key = `admin.analytics.learning.detail.domainEvidenceStatus.${status}`
  const label = t(key)
  return label === key ? status : label
}

export function LearningStudentDetailView({
  detail,
  isLoading,
  onBack,
}: {
  detail?: LearningStudentDetailResponse
  isLoading: boolean
  onBack: () => void
}) {
  const { t } = useTranslation()

  if (isLoading && !detail) {
    return <PageLoading />
  }

  const assessmentSummary = normalizeLearningAssessmentSummary(
    detail?.assessmentSummary
  )

  const metricCards: MetricCardsItem[] = [
    {
      key: "coverage",
      label: t("admin.analytics.learning.detail.metrics.coverage"),
      value: detail ? `${Math.round(detail.coverageRate * 100)}%` : "-",
      icon: <MessageSquareText className="size-4" />,
      tail: detail
        ? t("admin.analytics.learning.detail.metrics.coverageTail", {
            count: detail.observedKnowledgeCount,
          })
        : undefined,
    },
    {
      key: "assessed",
      label: t("admin.analytics.learning.detail.metrics.assessed"),
      value: detail ? String(assessmentSummary.assessedKnowledgeCount) : "-",
      icon: <ClipboardCheck className="size-4" />,
      tail: detail
        ? t("admin.analytics.learning.detail.metrics.assessmentTail", {
            count: assessmentSummary.assessmentCount,
          })
        : undefined,
    },
    {
      key: "demonstrated",
      label: t("admin.analytics.learning.detail.metrics.demonstrated"),
      value: detail
        ? String(assessmentSummary.demonstratedKnowledgeCount)
        : "-",
      icon: <BadgeCheck className="size-4" />,
      variant: "success",
    },
    {
      key: "retained",
      label: t("admin.analytics.learning.detail.metrics.retained"),
      value: detail ? String(assessmentSummary.retainedKnowledgeCount) : "-",
      icon: <History className="size-4" />,
    },
    {
      key: "needsAttention",
      label: t("admin.analytics.learning.detail.metrics.needsAttention"),
      value: detail
        ? String(assessmentSummary.needsAttentionKnowledgeCount)
        : "-",
      icon: <TriangleAlert className="size-4" />,
      variant: "warning",
    },
    {
      key: "tracking",
      label: t("admin.analytics.learning.detail.metrics.tracking"),
      value: detail
        ? detail.enabled
          ? t("admin.analytics.learning.detail.metrics.trackingOn")
          : t("admin.analytics.learning.detail.metrics.trackingOff")
        : "-",
      icon: <Clock3 className="size-4" />,
    },
  ]

  return (
    <div className="w-full min-w-0 space-y-4">
      <button
        type="button"
        onClick={onBack}
        className="inline-flex h-8 items-center gap-1.5 rounded-md text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
      >
        <ArrowLeft className="size-4" />
        {t("admin.analytics.learning.detail.back")}
      </button>

      <div className="min-w-0">
        <h1 className="text-2xl font-semibold tracking-normal">
          {t("admin.analytics.learning.detail.heading.title")}
        </h1>
        <p className="mt-1 text-sm text-(--app-muted-text)">
          {t("admin.analytics.learning.detail.heading.description")}
        </p>
        {detail ? (
          <p className="mt-2 text-sm text-foreground">
            {detail.displayName?.trim() || detail.displayId || detail.userId}
            {detail.schoolName || detail.teamName
              ? ` · ${detail.schoolName || detail.teamName}`
              : ""}
          </p>
        ) : null}
      </div>

      <MetricCards
        className="grid-cols-2 lg:grid-cols-3 xl:grid-cols-6"
        items={metricCards}
      />

      <section className="rounded-xl border border-(--app-border) bg-(--app-surface) p-4">
        <h2 className="text-sm font-semibold text-foreground">
          {t("admin.analytics.learning.detail.sections.mastery.title")}
        </h2>
        <p className="mt-1 text-xs text-muted-foreground">
          {t("admin.analytics.learning.detail.sections.mastery.description")}
        </p>
        {detail && detail.masterySummary.length > 0 ? (
          <div className="mt-3 grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
            {detail.masterySummary.map((entry) => (
              <div
                key={entry.knowledgeCode}
                className="flex items-center justify-between border border-(--app-border) px-3 py-2 text-sm"
              >
                <span className="text-foreground">
                  {getDomainLabel(t, entry.knowledgeCode)}
                </span>
                <span className="text-xs text-muted-foreground">
                  {getDomainEvidenceStatusLabel(t, entry.status)}
                  {entry.strength != null
                    ? ` · ${Math.round(entry.strength * 100)}%`
                    : ""}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <p className="mt-3 text-sm text-muted-foreground">
            {t("admin.analytics.learning.detail.sections.mastery.empty")}
          </p>
        )}
      </section>

      <LearningEvidenceList items={detail?.evidenceSummary ?? []} />
    </div>
  )
}
