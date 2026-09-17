import type { DataTableColumn } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import { Badge } from "@workspace/ui-components"
import type {
  LearningQuestionMetricRow,
  LearningQuestionReleaseStatus,
} from "../../../types"
import { LearningQuestionApplicabilityScopeCell } from "./applicability-scope-cell"
import { buildLearningChoiceDistributionColumn } from "./choice-distribution-column"

function formatRate(value: number) {
  return `${Math.round(value * 100)}%`
}

function releaseStatusBadgeVariant(status: LearningQuestionReleaseStatus) {
  switch (status) {
    case "production_approved":
      return "default" as const
    case "curriculum_pending":
      return "outline" as const
    case "internal_pilot":
      return "secondary" as const
    case "unknown":
      return "ghost" as const
  }
}

export function buildLearningQuestionColumns(
  t: TFunction
): DataTableColumn<LearningQuestionMetricRow>[] {
  return [
    {
      key: "knowledgeCode",
      header: t("admin.analytics.learning.questionTable.columns.domain"),
      width: 130,
      sortable: true,
      renderCell: (row) => (
        <Badge variant="outline">
          {t(`admin.analytics.learning.detail.domains.${row.knowledge_code}`)}
        </Badge>
      ),
    },
    {
      key: "questionId",
      header: t("admin.analytics.learning.questionTable.columns.question"),
      width: 420,
      sortable: true,
      renderCell: (row) => (
        <div className="min-w-72 max-w-xl whitespace-normal">
          <p className="text-sm/5 font-medium text-foreground">
            {row.question_text}
          </p>
          <p className="mt-1 break-all font-mono text-[11px]/4 text-muted-foreground">
            {row.question_id} · v{row.question_version}
          </p>
        </div>
      ),
    },
    {
      key: "applicabilityScope",
      header: t("admin.analytics.learning.questionTable.columns.applicabilityScope"),
      width: 230,
      sortable: true,
      renderCell: (row) => (
        <LearningQuestionApplicabilityScopeCell row={row} t={t} />
      ),
    },
    buildLearningChoiceDistributionColumn(t),
    {
      key: "releaseStatus",
      header: t("admin.analytics.learning.questionTable.columns.releaseStatus"),
      width: 140,
      sortable: true,
      renderCell: (row) => (
        <Badge variant={releaseStatusBadgeVariant(row.release_status)}>
          {t(
            `admin.analytics.learning.questionTable.releaseStatuses.${row.release_status}`
          )}
        </Badge>
      ),
    },
    {
      key: "sourcePublishers",
      header: t("admin.analytics.learning.questionTable.columns.sources"),
      width: 190,
      sortable: true,
      renderCell: (row) => (
        <p className="max-w-48 whitespace-normal text-xs/5 text-foreground">
          {row.source_publishers.length
            ? row.source_publishers.join(" / ")
            : t("admin.analytics.learning.questionTable.notAvailable")}
        </p>
      ),
    },
    {
      key: "sourceValidThrough",
      header: t("admin.analytics.learning.questionTable.columns.validThrough"),
      width: 130,
      sortable: true,
      renderCell: (row) =>
        row.source_valid_through ??
        t("admin.analytics.learning.questionTable.notAvailable"),
    },
    {
      key: "issuedCount",
      header: t("admin.analytics.learning.questionTable.columns.issued"),
      width: 150,
      sortable: true,
      renderCell: (row) => (
        <div>
          <p className="font-medium text-foreground">{row.issued_count}</p>
          <p className="text-xs text-muted-foreground">
            {t("admin.analytics.learning.questionTable.columns.issuedDetail", {
              active: row.active_count,
              superseded: row.superseded_count,
            })}
          </p>
        </div>
      ),
    },
    {
      key: "answeredCount",
      header: t("admin.analytics.learning.questionTable.columns.answered"),
      width: 100,
      sortable: true,
      renderCell: (row) => row.answered_count,
    },
    {
      key: "skippedCount",
      header: t("admin.analytics.learning.questionTable.columns.skipped"),
      width: 130,
      sortable: true,
      renderCell: (row) => (
        <div>
          <p className="font-medium text-foreground">{row.skipped_count}</p>
          <p className="text-xs text-muted-foreground">
            {t("admin.analytics.learning.questionTable.columns.expiredDetail", {
              expired: row.expired_count,
            })}
          </p>
        </div>
      ),
    },
    {
      key: "answerRate",
      header: t("admin.analytics.learning.questionTable.columns.answerRate"),
      width: 110,
      sortable: true,
      renderCell: (row) => formatRate(row.answer_rate),
    },
    {
      key: "correctRate",
      header: t("admin.analytics.learning.questionTable.columns.correctRate"),
      width: 140,
      sortable: true,
      renderCell: (row) => formatRate(row.correct_rate),
    },
    {
      key: "averageAnswerSeconds",
      header: t("admin.analytics.learning.questionTable.columns.averageSeconds"),
      width: 130,
      sortable: true,
      renderCell: (row) =>
        t("admin.analytics.learning.questionTable.columns.secondsValue", {
          count: row.average_answer_seconds,
        }),
    },
  ]
}
