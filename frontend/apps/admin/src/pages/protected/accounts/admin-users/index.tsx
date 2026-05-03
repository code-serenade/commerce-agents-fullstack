import { useCallback, useState } from "react"
import { DataTable, MetricCards } from "@workspace/app-kit"
import { useAdminUsersData } from "./admin-users-data"
import { useCreateAdminUserInsertAction } from "./dialogs/create-admin-user-dialog"
import { EditAdminUserRolesDialog } from "./dialogs/edit-admin-user-roles-dialog"
import { useAdminUsersTable } from "./table"
import type { AdminUserRow, AdminUserTableQuery } from "./types"

export default function AdminUsersPage() {
  const [roleDialogRow, setRoleDialogRow] = useState<AdminUserRow | null>(null)
  const [tableRefreshKey, setTableRefreshKey] = useState(0)
  const { metricCards, fetchData, invalidateAdminUsers } = useAdminUsersData()
  const handleEditRoles = useCallback((row: AdminUserRow) => {
    setRoleDialogRow(row)
  }, [])
  const table = useAdminUsersTable(
    fetchData,
    invalidateAdminUsers,
    handleEditRoles
  )
  const insertAction = useCreateAdminUserInsertAction(invalidateAdminUsers)

  return (
    <div className="w-full min-w-0 space-y-4">
      <MetricCards items={metricCards} />

      <div className="flex h-[calc(100vh-22rem)] min-h-160 min-w-0 flex-1 overflow-hidden">
        <DataTable<AdminUserRow, AdminUserTableQuery>
          key={tableRefreshKey}
          columns={table.columns}
          fetchData={table.fetchData}
          getRowId={table.getRowId}
          height="100%"
          initialPageSize={table.initialPageSize}
          initialQuery={table.initialQuery}
          insert={insertAction}
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

      <EditAdminUserRolesDialog
        onSaved={() => setTableRefreshKey((current) => current + 1)}
        open={Boolean(roleDialogRow)}
        row={roleDialogRow}
        onOpenChange={(open) => {
          if (!open) {
            setRoleDialogRow(null)
          }
        }}
      />
    </div>
  )
}
