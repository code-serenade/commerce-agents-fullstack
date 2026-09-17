import type { DataTableColumn } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { LearningQuestionMetricRow } from "../../../types"
import { LearningChoiceDistributionCell } from "./choice-distribution-cell"

export function buildLearningChoiceDistributionColumn(
  t: TFunction
): DataTableColumn<LearningQuestionMetricRow> {
  return {
    key: "choiceDistribution",
    header: t(
      "admin.analytics.learning.questionTable.columns.choiceDistribution"
    ),
    width: 360,
    renderCell: (row) => (
      <LearningChoiceDistributionCell
        choices={row.choice_metrics}
        unmappedChoiceCount={row.unmapped_choice_count}
      />
    ),
  }
}
