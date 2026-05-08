import { useQuery } from "@tanstack/react-query"
import {
  getAlertSummaryApi,
  getDailyBriefingSummaryApi,
  listAlertsApi,
  listBackgroundEventsApi,
  listDailyBriefingsApi,
  listLongTasksApi,
} from "@/api"

export const operationsQueryKey = ["commerce", "operations"]

export function useOperationQueries() {
  const longTasksQuery = useQuery({
    queryKey: [...operationsQueryKey, "long-tasks"],
    queryFn: () => listLongTasksApi({ pageSize: 100 }),
  })
  const dailySummaryQuery = useQuery({
    queryKey: [...operationsQueryKey, "daily-summary"],
    queryFn: () => getDailyBriefingSummaryApi(),
  })
  const dailyRecordsQuery = useQuery({
    queryKey: [...operationsQueryKey, "daily-records"],
    queryFn: () => listDailyBriefingsApi({ pageSize: 100 }),
  })
  const eventsQuery = useQuery({
    queryKey: [...operationsQueryKey, "notifications"],
    queryFn: () => listBackgroundEventsApi({ pageSize: 100 }),
  })
  const alertsQuery = useQuery({
    queryKey: [...operationsQueryKey, "alerts"],
    queryFn: () => listAlertsApi({ includeSnoozed: true, pageSize: 100 }),
  })
  const alertSummaryQuery = useQuery({
    queryKey: [...operationsQueryKey, "alert-summary"],
    queryFn: getAlertSummaryApi,
  })

  return {
    alertSummary: alertSummaryQuery.data,
    alerts: alertsQuery.data?.alerts ?? [],
    dailyRecords: dailyRecordsQuery.data?.records ?? [],
    dailySummary: dailySummaryQuery.data,
    loading:
      longTasksQuery.isLoading ||
      dailySummaryQuery.isLoading ||
      dailyRecordsQuery.isLoading ||
      eventsQuery.isLoading ||
      alertsQuery.isLoading ||
      alertSummaryQuery.isLoading,
    longTasks: longTasksQuery.data?.tasks ?? [],
    notifications: eventsQuery.data?.events ?? [],
  }
}
