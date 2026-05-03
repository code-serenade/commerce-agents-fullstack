import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { useTranslation } from "react-i18next"
import { buildAppUserColumns } from "./columns"
import {
  buildAppUserBuiltInQueryFields,
  buildAppUserQueryFields,
} from "./query-fields"
import {
  appUserInitialQuery,
  appUserPageSizeOptions,
} from "../constants"
import { useAppUserRowActions } from "./row-actions"
import type { AppUserRow, AppUserTableQuery } from "../types"

export function useAppUsersTable(
  fetchData: (params: {
    page: number
    pageSize: number
    query: AppUserTableQuery
    signal: AbortSignal
    sort: DataTableSortState | null
  }) => Promise<DataTableFetchResult<AppUserRow>>,
  invalidateAppUsers: () => Promise<unknown>,
  onEditRoles: (row: AppUserRow) => void
) {
  const { t } = useTranslation()
  const rowActions = useAppUserRowActions(invalidateAppUsers, onEditRoles)

  return {
    columns: buildAppUserColumns(t),
    fetchData,
    getRowId: (row: AppUserRow) => row.user_id,
    initialPageSize: 10,
    initialSort: {
      columnKey: "updatedAt",
      direction: "desc",
    },
    initialQuery: appUserInitialQuery,
    query: {
      builtInFields: buildAppUserBuiltInQueryFields(t),
      fields: buildAppUserQueryFields(t),
      audit: true,
    },
    pageSizeOptions: appUserPageSizeOptions,
    rowActions,
    bulkDelete: false,
    bulkUpdate: false,
    compactColumns: true,
    compactRows: true,
    fillWidth: false,
  } as const
}
