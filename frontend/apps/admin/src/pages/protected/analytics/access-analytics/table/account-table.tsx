import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import { CopyableText, DataTable, type DataTableColumn } from "@workspace/app-kit"
import type { DataTableFetchResult, DataTableSortState } from "@workspace/app-kit"
import {
  analyticsAccountInitialQuery,
  analyticsPageSizeOptions,
} from "../constants"
import { formatDateTime, formatInteger } from "../format"
import type { AnalyticsAccountRow, AnalyticsAccountTableQuery } from "../types"

interface AnalyticsAccountTableProps {
  fetchData: (params: {
    page: number
    pageSize: number
    query: AnalyticsAccountTableQuery
    signal: AbortSignal
    sort: DataTableSortState | null
  }) => Promise<DataTableFetchResult<AnalyticsAccountRow>>
  refreshKey: number
}

export function AnalyticsAccountTable({
  fetchData,
  refreshKey,
}: AnalyticsAccountTableProps) {
  const { t } = useTranslation()
  const columns = useMemo(() => buildColumns(t), [t])

  return (
    <DataTable<AnalyticsAccountRow, AnalyticsAccountTableQuery>
      key={refreshKey}
      columns={columns}
      fetchData={fetchData}
      getRowId={(row) => row.user_id}
      height="100%"
      initialPageSize={10}
      initialSort={{ columnKey: "lastSeenAt", direction: "desc" }}
      initialQuery={analyticsAccountInitialQuery}
      query={{
        builtInFields: [
          {
            key: "keyword",
            type: "search",
            label: t("admin.analytics.tables.accounts.query.keyword.label"),
            placeholder: t(
              "admin.analytics.tables.accounts.query.keyword.placeholder"
            ),
          },
        ],
        fields: [
          {
            key: "schoolName",
            type: "text",
            label: t("admin.analytics.tables.accounts.query.schoolName.label"),
            placeholder: t(
              "admin.analytics.tables.accounts.query.schoolName.placeholder"
            ),
          },
          {
            key: "teamName",
            type: "text",
            label: t("admin.analytics.tables.accounts.query.teamName.label"),
            placeholder: t(
              "admin.analytics.tables.accounts.query.teamName.placeholder"
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
): DataTableColumn<AnalyticsAccountRow>[] {
  return [
    {
      key: "display_id",
      header: t("admin.analytics.tables.accounts.columns.displayId"),
      width: 180,
      renderCell: (row) => (
        <CopyableText value={row.display_id} textClassName="text-sm">
          {row.display_id}
        </CopyableText>
      ),
    },
    {
      key: "display_name",
      header: t("admin.analytics.tables.accounts.columns.displayName"),
      width: 160,
      renderCell: (row) => <span className="font-medium">{row.display_name}</span>,
    },
    {
      key: "school_name",
      header: t("admin.analytics.tables.accounts.columns.schoolName"),
      width: 160,
      renderCell: (row) => row.school_name,
    },
    {
      key: "team_name",
      header: t("admin.analytics.tables.accounts.columns.teamName"),
      width: 150,
      renderCell: (row) => row.team_name,
    },
    {
      key: "pv",
      header: t("admin.analytics.tables.common.pv"),
      width: 100,
      sortable: true,
      renderCell: (row) => formatInteger(row.pv),
    },
    {
      key: "sessions",
      header: t("admin.analytics.tables.common.sessions"),
      width: 110,
      sortable: true,
      renderCell: (row) => formatInteger(row.sessions),
    },
    {
      key: "active_days",
      header: t("admin.analytics.tables.accounts.columns.activeDays"),
      width: 120,
      sortable: true,
      renderCell: (row) => formatInteger(row.active_days),
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
      key: "top_pages",
      header: t("admin.analytics.tables.accounts.columns.topPages"),
      width: 320,
      renderCell: (row) =>
        row.top_pages.length > 0 ? (
          <div className="space-y-1">
            {row.top_pages.slice(0, 3).map((page) => (
              <div key={page.path} className="min-w-0">
                <span className="font-medium">{page.label}</span>
                <span className="text-(--app-muted-text)">
                  {" "}
                  {t("admin.analytics.tables.accounts.topPagePv", {
                    pv: formatInteger(page.pv),
                  })}
                </span>
                <p className="truncate font-mono text-xs text-(--app-muted-text)">
                  {page.path}
                </p>
              </div>
            ))}
          </div>
        ) : (
          <span className="text-(--app-muted-text)">-</span>
        ),
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
