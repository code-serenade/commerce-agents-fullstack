import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { useTranslation } from "react-i18next"
import {
  roleInitialQuery,
  rolePageSizeOptions,
} from "../constants"
import { buildRoleColumns } from "./columns"
import { buildRoleBuiltInQueryFields } from "./query-fields"
import { useRoleRowActions } from "./row-actions"
import type { RoleRow, RoleTableQuery } from "../types"

export function useRolesTable(
  fetchData: (params: {
    page: number
    pageSize: number
    query: RoleTableQuery
    signal: AbortSignal
    sort: DataTableSortState | null
  }) => Promise<DataTableFetchResult<RoleRow>>,
  invalidateRoles: () => Promise<unknown>
) {
  const { t } = useTranslation()
  const rowActions = useRoleRowActions(invalidateRoles)

  return {
    columns: buildRoleColumns(t),
    fetchData,
    getRowId: (row: RoleRow) => row.id,
    initialPageSize: 10,
    initialQuery: roleInitialQuery,
    query: {
      builtInFields: buildRoleBuiltInQueryFields(t),
      fields: [],
      tools: false,
    },
    pageSizeOptions: rolePageSizeOptions,
    rowActions,
    bulkDelete: false,
    bulkUpdate: false,
    compactColumns: true,
    compactRows: true,
    fillWidth: false,
  } as const
}
