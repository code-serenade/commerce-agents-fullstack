import { useCallback, useState } from "react"
import { DataTable, MetricCards } from "@workspace/app-kit"
import { TeamMembersDialog } from "./dialogs/team-members-dialog"
import { useTeamUsageData } from "./team-usage-data"
import { useTeamUsageTable } from "./table"
import type { TeamUsageRow, TeamUsageTableQuery } from "./types"

export default function TeamUsagePage() {
  const [membersDialogRow, setMembersDialogRow] =
    useState<TeamUsageRow | null>(null)
  const [tableRefreshKey, setTableRefreshKey] = useState(0)
  const { metricCards, fetchData, invalidateTeamUsage } = useTeamUsageData()
  const handleViewMembers = useCallback((row: TeamUsageRow) => {
    setMembersDialogRow(row)
  }, [])
  const handleInvalidateTeamUsage = useCallback(async () => {
    await invalidateTeamUsage()
    setTableRefreshKey((current) => current + 1)
  }, [invalidateTeamUsage])
  const table = useTeamUsageTable(
    fetchData,
    handleInvalidateTeamUsage,
    handleViewMembers
  )

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
          compactColumns={table.compactColumns}
          compactRows={table.compactRows}
          fillWidth={table.fillWidth}
        />
      </div>

      <TeamMembersDialog
        open={Boolean(membersDialogRow)}
        row={membersDialogRow}
        onOpenChange={(open) => {
          if (!open) {
            setMembersDialogRow(null)
          }
        }}
      />
    </div>
  )
}
