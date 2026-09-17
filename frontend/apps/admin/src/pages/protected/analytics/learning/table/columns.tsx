import type { DataTableColumn } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import { Badge, Progress } from "@workspace/ui-components"
import { getLearningEvaluationStatus } from "../../learning-assessment"
import {
  getLearningEvaluationStatusBadgeVariant,
  getLearningEvaluationStatusLabel,
  getLearningTrackingStatusLabel,
} from "./status"
import type { LearningStudentRow } from "../types"

function formatRate(value: number) {
  return `${Math.round(value * 100)}%`
}

export function buildLearningColumns(
  t: TFunction
): DataTableColumn<LearningStudentRow>[] {
  return [
    {
      key: "student",
      header: t("admin.analytics.learning.table.student"),
      renderCell: (row) => (
        <div className="min-w-0">
          <p className="truncate font-medium text-foreground">
            {row.display_name}
          </p>
          <p className="truncate text-xs text-muted-foreground">
            {row.display_id}
          </p>
        </div>
      ),
    },
    {
      key: "school",
      header: t("admin.analytics.learning.table.school"),
      renderCell: (row) => row.school_name || row.team_name || "-",
    },
    {
      key: "coverageRate",
      header: t("admin.analytics.learning.table.coverageRate"),
      sortable: true,
      renderCell: (row) => (
        <div className="flex min-w-36 items-center gap-2">
          <div className="w-24">
            <Progress value={Math.round(row.coverage_rate * 100)} />
          </div>
          <span className="shrink-0 text-xs text-muted-foreground">
            {formatRate(row.coverage_rate)}
          </span>
        </div>
      ),
    },
    {
      key: "observedKnowledgeCount",
      header: t("admin.analytics.learning.table.observed"),
      sortable: true,
      renderCell: (row) => String(row.observed_count),
    },
    {
      key: "structuredAssessment",
      header: t("admin.analytics.learning.table.structuredAssessment"),
      renderCell: (row) => (
        <div className="min-w-30">
          <p className="text-sm font-medium text-foreground">
            {t("admin.analytics.learning.table.assessedDomains", {
              count: row.assessment_summary.assessedKnowledgeCount,
            })}
          </p>
          <p className="text-xs text-muted-foreground">
            {t("admin.analytics.learning.table.assessmentCount", {
              count: row.assessment_summary.assessmentCount,
            })}
          </p>
        </div>
      ),
    },
    {
      key: "evaluationStatus",
      header: t("admin.analytics.learning.table.evaluation"),
      renderCell: (row) => {
        const status = getLearningEvaluationStatus(row.assessment_summary)
        return (
          <Badge variant={getLearningEvaluationStatusBadgeVariant(status)}>
            {getLearningEvaluationStatusLabel(t, status)}
          </Badge>
        )
      },
    },
    {
      key: "enabled",
      header: t("admin.analytics.learning.table.enabled"),
      renderCell: (row) => getLearningTrackingStatusLabel(t, row.enabled),
    },
    {
      key: "lastAnalyzedAt",
      header: t("admin.analytics.learning.table.lastAnalyzedAt"),
      sortable: true,
      renderCell: (row) =>
        row.last_analyzed_at
          ? new Date(row.last_analyzed_at).toLocaleString()
          : "-",
    },
  ]
}
