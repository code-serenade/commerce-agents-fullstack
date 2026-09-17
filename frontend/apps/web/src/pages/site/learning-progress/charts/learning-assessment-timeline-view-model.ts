import type { LearningAssessmentTimelinePoint } from "@/api"

export function buildLearningAssessmentTimelineRows(
  timeline: LearningAssessmentTimelinePoint[]
) {
  return timeline.map((point) => ({
    period: formatPeriodStart(point.periodStart),
    assessed: point.assessedKnowledgeCount,
    demonstrated: point.demonstratedKnowledgeCount,
    retained: point.retainedKnowledgeCount,
  }))
}

function formatPeriodStart(value: string) {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value)
  if (!match) return value
  const [, year, month, day] = match
  const date = new Date(Date.UTC(Number(year), Number(month) - 1, Number(day)))
  return date.toLocaleDateString(undefined, { month: "numeric", day: "numeric" })
}
