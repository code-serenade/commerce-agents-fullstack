import { useTranslation } from "react-i18next"
import { Tabs, TabsList, TabsTrigger } from "@workspace/ui-components"
import type { LearningEvidenceRange } from "@/api"

export function LearningAnswerReviewControls({
  evidenceRange,
  onEvidenceRangeChange,
  onReviewOnlyChange,
  reviewOnly,
}: {
  evidenceRange: LearningEvidenceRange
  onEvidenceRangeChange: (range: LearningEvidenceRange) => void
  onReviewOnlyChange: (reviewOnly: boolean) => void
  reviewOnly: boolean
}) {
  const { t } = useTranslation()
  return (
    <div className="mt-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <Tabs
        value={reviewOnly ? "review" : "all"}
        onValueChange={(value) => {
          if (value === "review" || value === "all") {
            onReviewOnlyChange(value === "review")
          }
        }}
      >
        <TabsList>
          <TabsTrigger value="all">
            {t("learningProgress.portfolio.allAnswers")}
          </TabsTrigger>
          <TabsTrigger value="review">
            {t("learningProgress.portfolio.reviewAnswers")}
          </TabsTrigger>
        </TabsList>
      </Tabs>
      <Tabs
        value={evidenceRange}
        onValueChange={(value) => {
          if (value === "recent" || value === "30d" || value === "90d") {
            onEvidenceRangeChange(value)
          }
        }}
      >
        <TabsList>
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
  )
}
