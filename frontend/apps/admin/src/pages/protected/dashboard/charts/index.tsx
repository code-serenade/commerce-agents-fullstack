import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components/stable/card"
import {
  CategoryBarChart,
  DonutChart,
  TrendLineChart,
} from "@workspace/ui-components/stable/chart"
import { channelStats, revenueTrend, userSegments } from "../dashboard-data"

export function DashboardCharts() {
  const { t } = useTranslation()
  const localizedRevenueTrend = useMemo(
    () =>
      revenueTrend.map((item) => ({
        ...item,
        day: t(item.dayKey),
      })),
    [t]
  )
  const localizedUserSegments = useMemo(
    () =>
      userSegments.map((item) => ({
        key: item.key,
        label: t(item.labelKey),
        value: item.value,
      })),
    [t]
  )

  return (
    <div className="grid gap-4 xl:grid-cols-[1.25fr_0.75fr]">
      <Card>
        <CardHeader>
          <CardTitle>
            {t("admin.dashboard.charts.revenueTrend.title")}
          </CardTitle>
          <CardDescription>
            {t("admin.dashboard.charts.revenueTrend.description")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <TrendLineChart
            data={localizedRevenueTrend}
            xKey="day"
            series={[
              {
                key: "revenue",
                label: t("admin.dashboard.charts.series.revenue"),
              },
              {
                key: "orders",
                label: t("admin.dashboard.charts.series.orders"),
              },
            ]}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>
            {t("admin.dashboard.charts.userSegments.title")}
          </CardTitle>
          <CardDescription>
            {t("admin.dashboard.charts.userSegments.description")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <DonutChart data={localizedUserSegments} />
        </CardContent>
      </Card>
    </div>
  )
}

export function ChannelChart() {
  const { t } = useTranslation()
  const localizedChannelStats = useMemo(
    () =>
      channelStats.map((item) => ({
        channel: t(item.channelKey),
        value: item.value,
      })),
    [t]
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.charts.channels.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.charts.channels.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <CategoryBarChart
          data={localizedChannelStats}
          xKey="channel"
          valueKey="value"
          label={t("admin.dashboard.charts.series.channelShare")}
        />
      </CardContent>
    </Card>
  )
}
