import { useTranslation } from "react-i18next"
import { Badge, Tabs, TabsList, TabsTrigger } from "@workspace/ui-components"
import type { LearningEvidenceEntry, LearningEvidenceRange } from "@/api"
import { LearningEvidenceList } from "./learning-evidence-list"

export function LearningEvidenceHistory({
  evidence,
  evidenceError,
  evidenceLoading,
  evidenceRange,
  onEvidenceRangeChange,
  onOpenConversation,
}: {
  evidence?: LearningEvidenceEntry[]
  evidenceError: boolean
  evidenceLoading: boolean
  evidenceRange: LearningEvidenceRange
  onEvidenceRangeChange: (range: LearningEvidenceRange) => void
  onOpenConversation: (conversationId: string) => void
}) {
  const { t } = useTranslation()

  return (
    <section className="border-t border-(--agent-chat-soft-border)/60 pt-5">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h2 className="text-sm font-semibold text-foreground">
            {t("learningProgress.evidenceTitle")}
          </h2>
          <p className="mt-1 text-xs/5 text-muted-foreground">
            {t("learningProgress.evidenceDescription")}
          </p>
        </div>
        <Tabs
          value={evidenceRange}
          onValueChange={(value) => {
            if (isLearningEvidenceRange(value)) {
              onEvidenceRangeChange(value)
            }
          }}
        >
          <TabsList className="h-auto max-w-full flex-wrap">
            <TabsTrigger value="recent">
              {t("learningProgress.evidenceRanges.recent")}
            </TabsTrigger>
            <TabsTrigger value="30d">
              {t("learningProgress.evidenceRanges.days30")}
            </TabsTrigger>
            <TabsTrigger value="90d">
              {t("learningProgress.evidenceRanges.days90")}
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>
      <p className="mt-3 text-xs text-muted-foreground">
        {t("learningProgress.evidenceRangeSummary", {
          count: String(evidence?.length ?? 0),
          range: t(`learningProgress.evidenceRanges.${rangeLabelKey(evidenceRange)}`),
        })}
      </p>
      <div
        className="mt-3 flex flex-wrap gap-x-4 gap-y-2 text-xs text-muted-foreground"
        aria-label={t("learningProgress.evidenceLegendLabel")}
      >
        <span className="inline-flex items-center gap-1.5">
          <Badge variant="secondary">
            {t("learningProgress.evidenceLegend.assessed")}
          </Badge>
          {t("learningProgress.evidenceLegend.assessedMeaning")}
        </span>
        <span className="inline-flex items-center gap-1.5">
          <Badge variant="outline">
            {t("learningProgress.evidenceLegend.observed")}
          </Badge>
          {t("learningProgress.evidenceLegend.observedMeaning")}
        </span>
      </div>
      <LearningEvidenceList
        evidence={evidence}
        hasError={evidenceError}
        isLoading={evidenceLoading}
        onOpenConversation={onOpenConversation}
      />
    </section>
  )
}

function isLearningEvidenceRange(value: string): value is LearningEvidenceRange {
  return value === "recent" || value === "30d" || value === "90d"
}

function rangeLabelKey(range: LearningEvidenceRange) {
  if (range === "30d") return "days30"
  if (range === "90d") return "days90"
  return "recent"
}
