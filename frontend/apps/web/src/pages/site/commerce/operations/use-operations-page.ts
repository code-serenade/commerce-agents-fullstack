import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useOperationActions } from "./use-operation-actions"
import { buildOperationMetricCards } from "./operation-metrics"
import { useOperationQueries } from "./use-operation-queries"

export function useOperationsPage() {
  const { t } = useTranslation()
  const [dialog, setDialog] = useState<OperationDialogName | null>(null)
  const queries = useOperationQueries()
  const actions = useOperationActions({ setDialog, t })
  const metricCards = useMemo(
    () =>
      buildOperationMetricCards(t, {
        alerts: queries.alerts,
        dailyRecords: queries.dailyRecords,
        longTasks: queries.longTasks,
        notifications: queries.notifications,
      }),
    [
      queries.alerts,
      queries.dailyRecords,
      queries.longTasks,
      queries.notifications,
      t,
    ]
  )

  return {
    ...actions,
    ...queries,
    dialog,
    metricCards,
    setDialog,
  }
}

export type OperationDialogName =
  | "long-task"
  | "daily-briefing"
  | "notification"
  | "alert"

export type OperationsPageState = ReturnType<typeof useOperationsPage>
