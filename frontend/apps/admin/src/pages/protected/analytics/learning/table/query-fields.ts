import type {
  DataTableBuiltInQueryField,
} from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { LearningStudentTableQuery } from "../types"

export function buildLearningBuiltInQueryFields(
  t: TFunction
): DataTableBuiltInQueryField<LearningStudentTableQuery>[] {
  return [
    {
      key: "keyword",
      type: "search",
      label: t("admin.analytics.learning.table.query.keyword.label"),
      placeholder: t(
        "admin.analytics.learning.table.query.keyword.placeholder"
      ),
    },
  ]
}
