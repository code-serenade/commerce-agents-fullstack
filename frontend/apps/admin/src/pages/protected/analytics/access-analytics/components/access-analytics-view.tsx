import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { MetricCards } from "@workspace/app-kit"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@workspace/ui-components"
import { AnalyticsAccountTable } from "../table/account-table"
import { AnalyticsSchoolTeamTable } from "../table/school-team-table"
import { DateRangeControls } from "./date-range-controls"
import { OverviewCharts } from "./overview-charts"
import { TableGuide } from "./table-guide"
import type { useAccessAnalyticsData } from "../analytics-data"
import type { AnalyticsDateRange } from "../types"

type AnalyticsTab = "overview" | "accounts" | "schoolTeams"

interface AccessAnalyticsViewProps {
  analytics: ReturnType<typeof useAccessAnalyticsData>
  range: AnalyticsDateRange
  onRangeChange: (range: AnalyticsDateRange) => void
  onRefresh: () => Promise<void>
  tableRefreshKey: number
}

export function AccessAnalyticsView({
  analytics,
  range,
  onRangeChange,
  onRefresh,
  tableRefreshKey,
}: AccessAnalyticsViewProps) {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState<AnalyticsTab>("overview")
  const chartSeries = useMemo(
    () => [
      { key: "pv", label: t("admin.analytics.charts.series.pv") },
      { key: "uv", label: t("admin.analytics.charts.series.uv") },
      { key: "questions", label: t("admin.analytics.charts.series.questions") },
    ],
    [t]
  )

  return (
    <div className="w-full min-w-0 space-y-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
        <div className="min-w-0">
          <h1 className="text-2xl font-semibold tracking-normal">
            {t("admin.analytics.heading.title")}
          </h1>
          <p className="mt-1 text-sm text-(--app-muted-text)">
            {t("admin.analytics.heading.description")}
          </p>
          <p className="mt-1 text-xs text-(--app-muted-text)">
            {t("admin.analytics.filters.rangeSummary", {
              from: range.from,
              to: range.to,
            })}
          </p>
        </div>
        <DateRangeControls
          range={range}
          onRangeChange={onRangeChange}
          onRefresh={onRefresh}
        />
      </div>

      <MetricCards items={analytics.metricCards} />

      <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as AnalyticsTab)}>
        <TabsList>
          <TabsTrigger value="overview">
            {t("admin.analytics.tabs.overview")}
          </TabsTrigger>
          <TabsTrigger value="accounts">
            {t("admin.analytics.tabs.accounts")}
          </TabsTrigger>
          <TabsTrigger value="schoolTeams">
            {t("admin.analytics.tabs.schoolTeams")}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview">
          <OverviewCharts
            analytics={analytics}
            chartSeries={chartSeries}
          />
        </TabsContent>

        <TabsContent value="accounts">
          <TableGuide
            title={t("admin.analytics.guides.accounts.title")}
            description={t("admin.analytics.guides.accounts.description")}
          />
          <div className="mt-4 flex h-[calc(100vh-24rem)] min-h-160 min-w-0 flex-1 overflow-hidden">
            <AnalyticsAccountTable
              fetchData={analytics.fetchAccounts}
              refreshKey={tableRefreshKey}
            />
          </div>
        </TabsContent>

        <TabsContent value="schoolTeams">
          <TableGuide
            title={t("admin.analytics.guides.schoolTeams.title")}
            description={t("admin.analytics.guides.schoolTeams.description")}
          />
          <div className="mt-4 flex h-[calc(100vh-24rem)] min-h-160 min-w-0 flex-1 overflow-hidden">
            <AnalyticsSchoolTeamTable
              fetchData={analytics.fetchSchoolTeams}
              refreshKey={tableRefreshKey}
            />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}
