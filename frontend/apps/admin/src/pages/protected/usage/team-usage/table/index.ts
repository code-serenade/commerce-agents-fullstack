import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { useCallback, useMemo } from "react"
import { useTranslation } from "react-i18next"
import { teamUsageInitialQuery, teamUsagePageSizeOptions } from "../constants"
import {
  buildTeamUsageBuiltInQueryFields,
  buildTeamUsageQueryFields,
} from "./query-fields"
import { useTeamUsageRowActions } from "./row-actions"
import { buildTeamUsageColumns } from "./columns"
import type { TeamUsageRow, TeamUsageTableQuery } from "../types"

export function useTeamUsageTable(
  fetchData: (params: {
    page: number
    pageSize: number
    query: TeamUsageTableQuery
    signal: AbortSignal
    sort: DataTableSortState | null
  }) => Promise<DataTableFetchResult<TeamUsageRow>>,
  invalidateTeamUsage: () => Promise<unknown>
) {
  const { t } = useTranslation()
  const rowActions = useTeamUsageRowActions(invalidateTeamUsage)
  const columns = useMemo(() => buildTeamUsageColumns(t), [t])
  const builtInFields = useMemo(
    () => buildTeamUsageBuiltInQueryFields(t),
    [t]
  )
  const fields = useMemo(() => buildTeamUsageQueryFields(t), [t])
  const getRowId = useCallback((row: TeamUsageRow) => row.id, [])
  const initialSort = useMemo(
    () => ({
      columnKey: "updatedAt",
      direction: "desc" as const,
    }),
    []
  )
  const query = useMemo(
    () => ({
      builtInFields,
      fields,
      audit: false,
    }),
    [builtInFields, fields]
  )

  return {
    columns,
    fetchData,
    getRowId,
    initialPageSize: 10,
    initialSort,
    initialQuery: teamUsageInitialQuery,
    query,
    pageSizeOptions: teamUsagePageSizeOptions,
    rowActions,
    bulkDelete: false,
    bulkUpdate: false,
    compactColumns: true,
    compactRows: true,
    fillWidth: false,
  } as const
}
