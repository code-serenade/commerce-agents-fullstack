import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { useTranslation } from "react-i18next"
import { buildAdminUserColumns } from "./columns"
import {
  buildAdminUserBuiltInQueryFields,
  buildAdminUserQueryFields,
} from "./query-fields"
import { adminUserInitialQuery, adminUserPageSizeOptions } from "../constants"
import { useAdminUserRowActions } from "./row-actions"
import type { AdminUserRow, AdminUserTableQuery } from "../types"

export function useAdminUsersTable(
  fetchData: (params: {
    page: number
    pageSize: number
    query: AdminUserTableQuery
    signal: AbortSignal
    sort: DataTableSortState | null
  }) => Promise<DataTableFetchResult<AdminUserRow>>,
  invalidateAdminUsers: () => Promise<unknown>,
  onEditRoles: (row: AdminUserRow) => void
) {
  const { t } = useTranslation()
  const rowActions = useAdminUserRowActions(invalidateAdminUsers, onEditRoles)

  return {
    columns: buildAdminUserColumns(t),
    fetchData,
    getRowId: (row: AdminUserRow) => row.user_id,
    initialPageSize: 10,
    initialQuery: adminUserInitialQuery,
    query: {
      builtInFields: buildAdminUserBuiltInQueryFields(t),
      fields: buildAdminUserQueryFields(t),
    },
    pageSizeOptions: adminUserPageSizeOptions,
    rowActions,
    bulkDelete: false,
    bulkUpdate: false,
    compactColumns: true,
    compactRows: true,
    fillWidth: false,
  } as const
}
