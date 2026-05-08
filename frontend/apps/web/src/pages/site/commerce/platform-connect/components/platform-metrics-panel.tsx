import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type { PlatformSummaryItem, StoreMetricResponse } from "@/api"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { PlatformConnectPageState } from "../use-platform-connect-page"

export function PlatformMetricsPanel({ page }: { page: PlatformConnectPageState }) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <CommercePanel
        title={t("commerce.platform.panels.localMetrics")}
        description={t("commerce.platform.panels.localMetricsDesc")}
        actions={
          <Button size="sm" onClick={() => page.setDialog("metric")}>
            {t("commerce.platform.panels.saveMetric")}
          </Button>
        }
      >
        <CommerceTable
          loading={page.loading}
          items={page.metrics}
          emptyText={t("commerce.platform.empty.metrics")}
          getRowId={(item) => item.id}
          columns={metricColumns(t)}
        />
      </CommercePanel>
      <CommercePanel
        title={t("commerce.platform.panels.summary")}
        description={t("commerce.platform.panels.summaryDesc")}
      >
        <CommerceTable
          loading={page.loading}
          items={page.summaries}
          emptyText={t("commerce.platform.empty.summary")}
          getRowId={(item) => item.platform}
          columns={summaryColumns(t)}
        />
      </CommercePanel>
    </div>
  )
}

function metricColumns(t: CommerceTranslate): CommerceColumn<StoreMetricResponse>[] {
  return [
    {
      key: "metric",
      header: t("commerce.platform.columns.metric"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.metricKey}</div>
          <MutedText>{item.platform} / {item.metricDate}</MutedText>
        </div>
      ),
    },
    {
      key: "value",
      header: t("commerce.common.value"),
      render: (item) => item.metricValue,
    },
    {
      key: "source",
      header: t("commerce.common.source"),
      render: (item) => item.source || "-",
    },
  ]
}

function summaryColumns(t: CommerceTranslate): CommerceColumn<PlatformSummaryItem>[] {
  return [
    {
      key: "platform",
      header: t("commerce.common.platform"),
      render: (item) => item.display,
    },
    {
      key: "gmv30d",
      header: t("commerce.platform.columns.gmv30"),
      render: (item) => item.gmv30d ?? "-",
    },
    {
      key: "orders30d",
      header: t("commerce.platform.columns.orders30"),
      render: (item) => item.orders30d ?? "-",
    },
    {
      key: "roi",
      header: t("commerce.overview.columns.roi"),
      render: (item) => item.adRoi30d ?? "-",
    },
  ]
}
