import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import { DataTable, type DataTableColumn } from "@workspace/app-kit"
import type { DataTableFetchResult, DataTableSortState } from "@workspace/app-kit"
import {
  analyticsPageSizeOptions,
  analyticsSchoolTeamInitialQuery,
} from "../constants"
import { formatDateTime, formatInteger } from "../format"
import type {
  AnalyticsSchoolTeamRow,
  AnalyticsSchoolTeamTableQuery,
} from "../types"

interface AnalyticsSchoolTeamTableProps {
  fetchData: (params: {
    page: number
    pageSize: number
    query: AnalyticsSchoolTeamTableQuery
    signal: AbortSignal
    sort: DataTableSortState | null
  }) => Promise<DataTableFetchResult<AnalyticsSchoolTeamRow>>
  refreshKey: number
}

export function AnalyticsSchoolTeamTable({
  fetchData,
  refreshKey,
}: AnalyticsSchoolTeamTableProps) {
  const { t } = useTranslation()
  const columns = useMemo(() => buildColumns(t), [t])

  return (
    <DataTable<AnalyticsSchoolTeamRow, AnalyticsSchoolTeamTableQuery>
      key={refreshKey}
      columns={columns}
      fetchData={fetchData}
      getRowId={(row) => row.id}
      height="100%"
      initialPageSize={10}
      initialSort={{ columnKey: "lastSeenAt", direction: "desc" }}
      initialQuery={analyticsSchoolTeamInitialQuery}
      query={{
        builtInFields: [],
        fields: [
          {
            key: "schoolName",
            type: "text",
            label: t("admin.analytics.tables.schoolTeams.query.schoolName.label"),
            placeholder: t(
              "admin.analytics.tables.schoolTeams.query.schoolName.placeholder"
            ),
          },
          {
            key: "teamName",
            type: "text",
            label: t("admin.analytics.tables.schoolTeams.query.teamName.label"),
            placeholder: t(
              "admin.analytics.tables.schoolTeams.query.teamName.placeholder"
            ),
          },
        ],
        audit: false,
      }}
      bulkDelete={false}
      bulkUpdate={false}
      pageSizeOptions={analyticsPageSizeOptions}
      stickyColumns={{ left: 1 }}
      compactColumns
      compactRows
      fillWidth={false}
    />
  )
}

function buildColumns(
  t: ReturnType<typeof useTranslation>["t"]
): DataTableColumn<AnalyticsSchoolTeamRow>[] {
  return [
    {
      key: "school_name",
      header: t("admin.analytics.tables.schoolTeams.columns.schoolName"),
      width: 180,
      renderCell: (row) => row.school_name,
    },
    {
      key: "team_name",
      header: t("admin.analytics.tables.schoolTeams.columns.teamName"),
      width: 180,
      renderCell: (row) => row.team_name,
    },
    {
      key: "account_count",
      header: t("admin.analytics.tables.schoolTeams.columns.accountCount"),
      width: 120,
      renderCell: (row) => formatInteger(row.account_count),
    },
    {
      key: "active_accounts",
      header: t("admin.analytics.tables.schoolTeams.columns.activeAccounts"),
      width: 130,
      sortable: true,
      renderCell: (row) => formatInteger(row.active_accounts),
    },
    {
      key: "pv",
      header: t("admin.analytics.tables.common.pv"),
      width: 100,
      sortable: true,
      renderCell: (row) => formatInteger(row.pv),
    },
    {
      key: "uv",
      header: t("admin.analytics.tables.common.uv"),
      width: 100,
      renderCell: (row) => formatInteger(row.uv),
    },
    {
      key: "sessions",
      header: t("admin.analytics.tables.common.sessions"),
      width: 110,
      sortable: true,
      renderCell: (row) => formatInteger(row.sessions),
    },
    {
      key: "agent_questions",
      header: t("admin.analytics.tables.common.agentQuestions"),
      width: 130,
      sortable: true,
      renderCell: (row) => formatInteger(row.agent_questions),
    },
    {
      key: "token_used",
      header: t("admin.analytics.tables.common.tokenUsed"),
      width: 130,
      renderCell: (row) => formatInteger(row.token_used),
    },
    {
      key: "lastSeenAt",
      header: t("admin.analytics.tables.common.lastSeenAt"),
      width: 180,
      sortable: true,
      renderCell: (row) => formatDateTime(row.lastSeenAt),
    },
  ]
}
