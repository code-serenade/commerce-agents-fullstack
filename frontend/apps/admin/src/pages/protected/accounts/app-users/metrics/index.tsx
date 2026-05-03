import { ShieldCheck, ShieldOff, SquareUserRound, Users } from "lucide-react"
import type { MetricCardsItem } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { AppUserMetricsResponse } from "@/api"

export function buildAppUserMetricCards(
  metrics: AppUserMetricsResponse | undefined,
  t: TFunction
): MetricCardsItem[] {
  return [
    {
      key: "all",
      label: t("admin.accounts.appUsers.metrics.all.label"),
      value: `${metrics?.total ?? 0}`,
      tail: t("admin.accounts.appUsers.metrics.all.tail"),
      icon: <SquareUserRound className="size-4" />,
    },
    {
      key: "enabled",
      label: t("admin.accounts.appUsers.metrics.enabled.label"),
      value: `${metrics?.enabled ?? 0}`,
      tail: t("admin.accounts.appUsers.metrics.enabled.tail"),
      icon: <ShieldCheck className="size-4" />,
      variant: "success",
    },
    {
      key: "disabled",
      label: t("admin.accounts.appUsers.metrics.disabled.label"),
      value: `${metrics?.disabled ?? 0}`,
      tail: t("admin.accounts.appUsers.metrics.disabled.tail"),
      icon: <ShieldOff className="size-4" />,
      variant: "danger",
    },
    {
      key: "multi-role",
      label: t("admin.accounts.appUsers.metrics.multiRole.label"),
      value: `${metrics?.multiRole ?? 0}`,
      tail: t("admin.accounts.appUsers.metrics.multiRole.tail"),
      icon: <Users className="size-4" />,
      variant: "accent",
    },
  ]
}
