import { Ban, Gauge, MessageSquareText, UsersRound } from "lucide-react"
import type { MetricCardsItem } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { TeamUsageRow } from "../types"

export function buildTeamUsageMetricCards(
  rows: TeamUsageRow[],
  total: number,
  t: TFunction
): MetricCardsItem[] {
  const disabled = rows.filter((row) => row.status === "disabled").length
  const exhausted = rows.filter(
    (row) => row.status === "enabled" && row.question_remaining <= 0
  ).length
  const questionUsed = rows.reduce(
    (sum, row) => sum + Math.max(0, row.question_used),
    0
  )

  return [
    {
      key: "all",
      label: t("admin.usage.teamUsage.metrics.all.label"),
      value: `${total}`,
      tail: t("admin.usage.teamUsage.metrics.all.tail"),
      icon: <UsersRound className="size-4" />,
    },
    {
      key: "question-used",
      label: t("admin.usage.teamUsage.metrics.questionUsed.label"),
      value: `${questionUsed}`,
      tail: t("admin.usage.teamUsage.metrics.questionUsed.tail"),
      icon: <MessageSquareText className="size-4" />,
      variant: "accent",
    },
    {
      key: "exhausted",
      label: t("admin.usage.teamUsage.metrics.exhausted.label"),
      value: `${exhausted}`,
      tail: t("admin.usage.teamUsage.metrics.exhausted.tail"),
      icon: <Gauge className="size-4" />,
      variant: exhausted > 0 ? "warning" : "success",
    },
    {
      key: "disabled",
      label: t("admin.usage.teamUsage.metrics.disabled.label"),
      value: `${disabled}`,
      tail: t("admin.usage.teamUsage.metrics.disabled.tail"),
      icon: <Ban className="size-4" />,
      variant: disabled > 0 ? "danger" : "default",
    },
  ]
}
