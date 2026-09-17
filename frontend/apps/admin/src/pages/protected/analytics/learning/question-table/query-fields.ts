import type { DataTableBuiltInQueryField } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { LearningQuestionTableQuery } from "../types"

const domainCodes = [
  "market",
  "product",
  "content",
  "traffic",
  "conversion",
  "finance",
  "service",
  "compliance",
] as const

export function buildLearningQuestionQueryFields(
  t: TFunction
): DataTableBuiltInQueryField<LearningQuestionTableQuery>[] {
  return [
    {
      key: "keyword",
      type: "search",
      label: t("admin.analytics.learning.questionTable.query.keyword.label"),
      placeholder: t(
        "admin.analytics.learning.questionTable.query.keyword.placeholder"
      ),
    },
    {
      key: "knowledgeCode",
      type: "select",
      label: t("admin.analytics.learning.questionTable.query.domain.label"),
      placeholder: t(
        "admin.analytics.learning.questionTable.query.domain.placeholder"
      ),
      options: domainCodes.map((code) => ({
        value: code,
        label: t(`admin.analytics.learning.detail.domains.${code}`),
      })),
    },
    {
      key: "releaseStatus",
      type: "select",
      label: t("admin.analytics.learning.questionTable.query.releaseStatus.label"),
      placeholder: t(
        "admin.analytics.learning.questionTable.query.releaseStatus.placeholder"
      ),
      options: [
        "production_approved",
        "curriculum_pending",
        "internal_pilot",
        "unknown",
      ].map((status) => ({
        value: status,
        label: t(
          `admin.analytics.learning.questionTable.releaseStatuses.${status}`
        ),
      })),
    },
  ]
}
