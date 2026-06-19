import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
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
  invalidateTeamUsage: () => Promise<unknown>,
  onViewMembers: (row: TeamUsageRow) => void
) {
  const { t } = useTranslation()
  const rowActions = useTeamUsageRowActions(
    invalidateTeamUsage,
    onViewMembers
  )

  return {
    columns: buildTeamUsageColumns(t),
    fetchData,
    getRowId: (row: TeamUsageRow) => row.id,
    initialPageSize: 10,
    initialSort: {
      columnKey: "updatedAt",
      direction: "desc",
    },
    initialQuery: teamUsageInitialQuery,
    query: {
      builtInFields: buildTeamUsageBuiltInQueryFields(t),
      fields: buildTeamUsageQueryFields(t),
      audit: false,
    },
    pageSizeOptions: teamUsagePageSizeOptions,
    rowActions,
    bulkDelete: false,
    bulkUpdate: false,
    compactColumns: true,
    compactRows: true,
    fillWidth: false,
  } as const
}
