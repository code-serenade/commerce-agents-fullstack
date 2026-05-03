import { DataTable } from "@workspace/app-kit"
import { useAppRolesData } from "./app-roles-data"
import { RoleCreationNote } from "./components/role-creation-note"
import { useCreateRoleInsertAction } from "./dialogs/create-role-dialog"
import { useRolesTable } from "./table"
import type { RoleRow, RoleTableQuery } from "./types"

export default function AppRolesPage() {
  const { fetchData, invalidateRoles } = useAppRolesData()
  const table = useRolesTable(fetchData, invalidateRoles)
  const insertAction = useCreateRoleInsertAction(invalidateRoles)

  return (
    <div className="w-full min-w-0 space-y-4">
      <RoleCreationNote />

      <div className="min-w-0 overflow-hidden">
        <DataTable<RoleRow, RoleTableQuery>
          columns={table.columns}
          fetchData={table.fetchData}
          getRowId={table.getRowId}
          initialPageSize={table.initialPageSize}
          initialQuery={table.initialQuery}
          insert={insertAction}
          query={table.query}
          pageSizeOptions={table.pageSizeOptions}
          bulkDelete={table.bulkDelete}
          bulkUpdate={table.bulkUpdate}
          rowActions={table.rowActions}
          compactColumns={table.compactColumns}
          compactRows={table.compactRows}
          fillWidth={table.fillWidth}
        />
      </div>
    </div>
  )
}
