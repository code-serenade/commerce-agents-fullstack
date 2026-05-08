import type { MetricCardsItem } from "@workspace/app-kit"
import type {
  AlertResponse,
  BackgroundEventResponse,
  DailyBriefingResponse,
  LongTaskResponse,
} from "@/api"
import type { CommerceTranslate } from "../components/commerce-types"

export function buildOperationMetricCards(
  t: CommerceTranslate,
  data: {
    alerts: AlertResponse[]
    dailyRecords: DailyBriefingResponse[]
    longTasks: LongTaskResponse[]
    notifications: BackgroundEventResponse[]
  }
): MetricCardsItem[] {
  return [
    {
      key: "long-tasks",
      label: t("commerce.operations.metrics.longTasks"),
      value: data.longTasks.length,
      tail: t("commerce.common.countRunning", {
        count: data.longTasks.filter((item) => item.status === "running")
          .length,
      }),
      variant: "accent",
    },
    {
      key: "briefings",
      label: t("commerce.operations.metrics.briefings"),
      value: data.dailyRecords.length,
      tail: t("commerce.operations.metrics.briefingsTail"),
      variant: "success",
    },
    {
      key: "notifications",
      label: t("commerce.operations.metrics.notifications"),
      value: data.notifications.length,
      tail: t("commerce.common.countUnread", {
        count: data.notifications.filter((item) => !item.consumed).length,
      }),
      variant: "default",
    },
    {
      key: "alerts",
      label: t("commerce.operations.metrics.alerts"),
      value: data.alerts.length,
      tail: t("commerce.common.countActive", {
        count: data.alerts.filter((item) => item.status === "active").length,
      }),
      variant: "warning",
    },
  ]
}
