import { useTranslation } from "react-i18next"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components/stable/card"
import { TrendLineChart } from "@workspace/ui-components/stable/chart"
import type { DashboardProductMetric } from "../dashboard-types"

export function DashboardCharts({
  productMix,
  trend,
}: {
  productMix: DashboardProductMetric[]
  trend: Array<Record<string, number | string>>
}) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 xl:grid-cols-[1.25fr_0.75fr]">
      <Card>
        <CardHeader>
          <CardTitle>
            {t("admin.dashboard.charts.trafficTrend.title")}
          </CardTitle>
          <CardDescription>
            {t("admin.dashboard.charts.trafficTrend.description")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <TrendLineChart
            data={trend}
            xKey="date"
            series={[
              {
                key: "pv",
                label: t("admin.dashboard.charts.series.pv"),
              },
              {
                key: "uv",
                label: t("admin.dashboard.charts.series.uv"),
              },
              {
                key: "questions",
                label: t("admin.dashboard.charts.series.questions"),
              },
            ]}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>
            {t("admin.dashboard.charts.productMix.title")}
          </CardTitle>
          <CardDescription>
            {t("admin.dashboard.charts.productMix.description")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-3">
            {productMix.map((item) => (
              <div
                key={item.key}
                className="flex items-center justify-between gap-3 rounded-(--ui-radius-md) bg-(--app-active-bg) px-3 py-2"
              >
                <span className="text-sm text-(--app-muted-text)">
                  {item.label}
                </span>
                <span className="text-base font-semibold">{item.value}</span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
