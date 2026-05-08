import type { MetricCardsItem } from "@workspace/app-kit"
import type {
  ChannelConnectionResponse,
  PlatformConnectionResponse,
  PlatformSummaryItem,
  StoreMetricResponse,
} from "@/api"
import type { CommerceTranslate } from "../components/commerce-types"

export function buildPlatformMetricCards(
  t: CommerceTranslate,
  data: {
    channelConnections: ChannelConnectionResponse[]
    metrics: StoreMetricResponse[]
    platformConnections: PlatformConnectionResponse[]
    summaries: PlatformSummaryItem[]
  }
): MetricCardsItem[] {
  const gmv30d = data.summaries.reduce(
    (sum, item) => sum + (item.gmv30d ?? 0),
    0
  )

  return [
    {
      key: "platforms",
      label: t("commerce.platform.metrics.platforms"),
      value: data.platformConnections.length,
      tail: t("commerce.common.countEnabled", {
        count: data.platformConnections.filter((item) => item.enabled).length,
      }),
      variant: "accent",
    },
    {
      key: "channels",
      label: t("commerce.platform.metrics.channels"),
      value: data.channelConnections.length,
      tail: t("commerce.common.countEnabled", {
        count: data.channelConnections.filter((item) => item.enabled).length,
      }),
      variant: "success",
    },
    {
      key: "metrics",
      label: t("commerce.platform.metrics.metrics"),
      value: data.metrics.length,
      tail: t("commerce.platform.metrics.metricsTail"),
      variant: "default",
    },
    {
      key: "gmv",
      label: t("commerce.platform.metrics.gmv"),
      value: gmv30d,
      tail: t("commerce.platform.metrics.gmvTail"),
      variant: "warning",
    },
  ]
}
