import type { TFunction } from "i18next"
import type { LearningEvaluationStatus } from "../../learning-assessment"

export function getLearningTrackingStatusLabel(
  t: TFunction,
  enabled: boolean
) {
  return enabled
    ? t("admin.analytics.learning.table.enabledYes")
    : t("admin.analytics.learning.table.enabledNo")
}

export function getLearningEvaluationStatusLabel(
  t: TFunction,
  status: LearningEvaluationStatus
) {
  return t(`admin.analytics.learning.table.evaluationStatus.${status}`)
}

export function getLearningEvaluationStatusBadgeVariant(
  status: LearningEvaluationStatus
) {
  switch (status) {
    case "needs_attention":
      return "destructive" as const
    case "retained":
      return "default" as const
    case "demonstrated":
      return "secondary" as const
    case "accumulating":
    case "unassessed":
      return "outline" as const
  }
}
