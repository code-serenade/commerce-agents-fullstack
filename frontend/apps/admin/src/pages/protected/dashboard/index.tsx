import { DashboardCharts } from "./charts"
import { DataStatusTable } from "./components/data-status-table"
import { DashboardHeading } from "./components/dashboard-heading"
import { SchoolTeamActivity } from "./components/school-team-activity"
import { TopPagesTable } from "./components/top-pages-table"
import { UsageAlertsTable } from "./components/usage-alerts-table"
import { useDashboardData } from "./dashboard-data"
import { DashboardMetrics } from "./metrics"

export default function DashboardPage() {
  const dashboard = useDashboardData()

  return (
    <div className="space-y-4">
      <DashboardHeading range={dashboard.range} isLoading={dashboard.isLoading} />
      <DashboardMetrics items={dashboard.metricCards} />
      <DashboardCharts productMix={dashboard.productMix} trend={dashboard.trend} />

      <div className="grid gap-4 xl:grid-cols-[0.9fr_1.1fr]">
        <SchoolTeamActivity rows={dashboard.schoolTeams} />
        <TopPagesTable pages={dashboard.topPages} />
      </div>

      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <UsageAlertsTable rows={dashboard.usageAlerts} />
        <DataStatusTable rows={dashboard.statusRows} />
      </div>
    </div>
  )
}
