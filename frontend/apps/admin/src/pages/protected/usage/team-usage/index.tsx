import { useCallback, useState } from "react"
import { DataTable, MetricCards } from "@workspace/app-kit"
import { useTeamUsageData } from "./team-usage-data"
import { useTeamUsageTable } from "./table"
import type { TeamUsageRow, TeamUsageTableQuery } from "./types"

export default function TeamUsagePage() {
  const [tableRefreshKey, setTableRefreshKey] = useState(0)
  const { metricCards, fetchData, invalidateTeamUsage } = useTeamUsageData()
  const handleInvalidateTeamUsage = useCallback(async () => {
    await invalidateTeamUsage()
    setTableRefreshKey((current) => current + 1)
  }, [invalidateTeamUsage])
  const table = useTeamUsageTable(fetchData, handleInvalidateTeamUsage)

  return (
    <div className="w-full min-w-0 space-y-4">
      <MetricCards items={metricCards} />

      <div className="flex h-[calc(100vh-22rem)] min-h-160 min-w-0 flex-1 overflow-hidden">
        <DataTable<TeamUsageRow, TeamUsageTableQuery>
          key={tableRefreshKey}
          columns={table.columns}
          fetchData={table.fetchData}
          getRowId={table.getRowId}
          height="100%"
          initialPageSize={table.initialPageSize}
          initialSort={table.initialSort}
          initialQuery={table.initialQuery}
          query={table.query}
          bulkDelete={table.bulkDelete}
          bulkUpdate={table.bulkUpdate}
          pageSizeOptions={table.pageSizeOptions}
          rowActions={table.rowActions}
          stickyColumns={{ left: 1 }}
          compactColumns={table.compactColumns}
          compactRows={table.compactRows}
          fillWidth={table.fillWidth}
        />
      </div>
    </div>
  )
}
