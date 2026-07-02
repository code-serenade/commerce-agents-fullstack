import { useTranslation } from "react-i18next"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components"
import {
  CategoryBarChart,
  TrendLineChart,
} from "@workspace/ui-components/stable/chart"
import { TopPagesTable } from "./top-pages-table"
import type { useAccessAnalyticsData } from "../analytics-data"

interface OverviewChartsProps {
  analytics: ReturnType<typeof useAccessAnalyticsData>
  chartSeries: Array<{ key: string; label: string }>
}

export function OverviewCharts({
  analytics,
  chartSeries,
}: OverviewChartsProps) {
  const { t } = useTranslation()

  return (
    <div className="mt-4 space-y-4">
      <div className="grid gap-4 xl:grid-cols-[1.25fr_0.75fr]">
        <Card>
          <CardHeader>
            <CardTitle>{t("admin.analytics.charts.traffic.title")}</CardTitle>
            <CardDescription>
              {t("admin.analytics.charts.traffic.description")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <TrendLineChart
              data={analytics.trafficTrend}
              xKey="date"
              series={chartSeries}
              height={280}
            />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>{t("admin.analytics.charts.sources.title")}</CardTitle>
            <CardDescription>
              {t("admin.analytics.charts.sources.description")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <CategoryBarChart
              data={analytics.sourceKindChart}
              xKey="source"
              valueKey="pv"
              label={t("admin.analytics.charts.series.pv")}
              height={280}
            />
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 xl:grid-cols-[0.8fr_1.2fr]">
        <Card>
          <CardHeader>
            <CardTitle>{t("admin.analytics.charts.pages.title")}</CardTitle>
            <CardDescription>
              {t("admin.analytics.charts.pages.description")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <CategoryBarChart
              data={analytics.pageChart}
              xKey="page"
              valueKey="pv"
              label={t("admin.analytics.charts.series.pv")}
              height={280}
            />
          </CardContent>
        </Card>
        <TopPagesTable pages={analytics.pages} />
      </div>
    </div>
  )
}
