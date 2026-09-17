import { useTranslation } from "react-i18next"
import { TrendLineChart } from "@workspace/ui-components/stable/chart"
import type { LearningAssessmentTimelinePoint } from "@/api"
import { buildLearningAssessmentTimelineRows } from "./learning-assessment-timeline-view-model"

export function LearningAssessmentTimeline({
  timeline,
}: {
  timeline?: LearningAssessmentTimelinePoint[]
}) {
  const { t } = useTranslation()

  if (!timeline) return null

  const rows = buildLearningAssessmentTimelineRows(timeline)
  const hasAssessmentEvidence = timeline.some(
    (point) =>
      point.assessedKnowledgeCount > 0 ||
      point.demonstratedKnowledgeCount > 0 ||
      point.retainedKnowledgeCount > 0
  )

  return (
    <section className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl">
      <h2 className="text-sm font-semibold text-foreground">
        {t("learningProgress.assessmentTimeline.title")}
      </h2>
      <p className="mt-1 text-xs/5 text-muted-foreground">
        {t("learningProgress.assessmentTimeline.description")}
      </p>
      {hasAssessmentEvidence ? (
        <div className="mt-3 min-w-0" data-testid="learning-assessment-timeline-chart">
          <TrendLineChart
            data={rows}
            height={220}
            series={[
              {
                key: "assessed",
                label: t("learningProgress.assessmentTimeline.assessed"),
              },
              {
                key: "demonstrated",
                label: t("learningProgress.assessmentTimeline.demonstrated"),
              },
              {
                key: "retained",
                label: t("learningProgress.assessmentTimeline.retained"),
              },
            ]}
            xKey="period"
          />
        </div>
      ) : (
        <p className="mt-3 text-sm/6 text-muted-foreground">
          {t("learningProgress.assessmentTimeline.empty")}
        </p>
      )}
    </section>
  )
}
