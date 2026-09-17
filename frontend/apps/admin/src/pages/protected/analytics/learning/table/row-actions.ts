import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import type { DataTableRowActionsConfig } from "@workspace/app-kit"
import type { LearningStudentRow } from "../types"

export function useLearningRowActions(): DataTableRowActionsConfig<LearningStudentRow> {
  const { t } = useTranslation()
  const navigate = useNavigate()

  return {
    moreItems: [
      {
        key: "view-detail",
        label: t("admin.analytics.learning.table.actions.viewDetail"),
        onClick: (row) => navigate(`/analytics/learning/${row.user_id}`),
      },
    ],
  }
}
