import { ShieldCheck, ShieldOff, SquareUserRound, Users } from "lucide-react"
import type { MetricCardsItem } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type { AdminUserRow } from "../types"

export function buildAdminUserMetricCards(
  rows: readonly AdminUserRow[],
  t: TFunction
): MetricCardsItem[] {
  return [
    {
      key: "all",
      label: t("admin.accounts.adminUsers.metrics.all.label"),
      value: `${rows.length}`,
      tail: t("admin.accounts.adminUsers.metrics.all.tail"),
      icon: <SquareUserRound className="size-4" />,
    },
    {
      key: "enabled",
      label: t("admin.accounts.adminUsers.metrics.enabled.label"),
      value: `${rows.filter((row) => row.status === "enabled").length}`,
      tail: t("admin.accounts.adminUsers.metrics.enabled.tail"),
      icon: <ShieldCheck className="size-4" />,
      variant: "success",
    },
    {
      key: "disabled",
      label: t("admin.accounts.adminUsers.metrics.disabled.label"),
      value: `${rows.filter((row) => row.status === "disabled").length}`,
      tail: t("admin.accounts.adminUsers.metrics.disabled.tail"),
      icon: <ShieldOff className="size-4" />,
      variant: "danger",
    },
    {
      key: "multi-role",
      label: t("admin.accounts.adminUsers.metrics.multiRole.label"),
      value: `${rows.filter((row) => row.roles.length > 1).length}`,
      tail: t("admin.accounts.adminUsers.metrics.multiRole.tail"),
      icon: <Users className="size-4" />,
      variant: "accent",
    },
  ]
}
