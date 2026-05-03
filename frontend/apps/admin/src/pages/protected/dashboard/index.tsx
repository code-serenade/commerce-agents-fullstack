import { ChannelChart, DashboardCharts } from "./charts"
import { DashboardHeading } from "./components/dashboard-heading"
import { OrderFlowTable } from "./components/order-flow-table"
import { RiskMonitor } from "./components/risk-monitor"
import { ServiceStatusTable } from "./components/service-status-table"
import { DashboardMetrics } from "./metrics"

export default function DashboardPage() {
  return (
    <div className="space-y-4">
      <DashboardHeading />
      <DashboardMetrics />
      <DashboardCharts />

      <div className="grid gap-4 xl:grid-cols-[0.9fr_1.1fr]">
        <ChannelChart />
        <OrderFlowTable />
      </div>

      <div className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <RiskMonitor />
        <ServiceStatusTable />
      </div>
    </div>
  )
}
