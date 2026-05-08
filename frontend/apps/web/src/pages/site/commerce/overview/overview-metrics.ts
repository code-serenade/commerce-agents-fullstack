import type { MetricCardsItem } from "@workspace/app-kit"
import type { CommerceTranslate } from "../components/commerce-types"
import type { CommerceOverviewData } from "./use-commerce-overview-page"

export function buildOverviewMetricCards(
  t: CommerceTranslate,
  data: CommerceOverviewData | undefined
): MetricCardsItem[] {
  return [
    {
      key: "products",
      label: t("commerce.overview.metrics.products"),
      value: data?.products.length ?? 0,
      tail: t("commerce.overview.metrics.productsTail"),
      variant: "accent",
    },
    {
      key: "campaigns",
      label: t("commerce.overview.metrics.campaigns"),
      value: data?.campaigns.length ?? 0,
      tail: t("commerce.overview.metrics.campaignsTail"),
      variant: "default",
    },
    {
      key: "workspaces",
      label: t("commerce.overview.metrics.workspaces"),
      value: data?.workspaces.length ?? 0,
      tail: t("commerce.overview.metrics.workspacesTail"),
      variant: "success",
    },
    {
      key: "risk",
      label: t("commerce.overview.metrics.pending"),
      value:
        (data?.executionQueue.length ?? 0) +
        (data?.approvals.length ?? 0) +
        (data?.alerts.active ?? 0),
      tail: t("commerce.overview.metrics.pendingTail"),
      variant: "warning",
    },
  ]
}
