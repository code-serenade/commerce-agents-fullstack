import { useState } from "react"
import { useAccessAnalyticsData } from "./analytics-data"
import { AccessAnalyticsView } from "./components/access-analytics-view"
import { defaultDateRange } from "./format"
import type { AnalyticsDateRange } from "./types"

export default function AccessAnalyticsPage() {
  const [range, setRange] = useState<AnalyticsDateRange>(() => defaultDateRange())
  const [tableRefreshKey, setTableRefreshKey] = useState(0)
  const analytics = useAccessAnalyticsData(range)

  const refresh = async () => {
    await analytics.refresh()
    setTableRefreshKey((current) => current + 1)
  }

  return (
    <AccessAnalyticsView
      analytics={analytics}
      range={range}
      onRangeChange={setRange}
      onRefresh={refresh}
      tableRefreshKey={tableRefreshKey}
    />
  )
}
