import { useTranslation } from "react-i18next"
import { DataTable, type DataTableColumn } from "@workspace/app-kit"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components"
import { formatDuration, formatInteger } from "../format"
import { resolveRouteDisplayInfo } from "../../route-labels"
import type { RouteDisplayInfo } from "../../route-labels"

interface TopPagesTableProps {
  pages: TopPageMetric[]
}

interface TopPageMetric {
  routePath: string
  pv: number
  uv: number
  sessions: number
  avgSessionDurationSec: number
  entryCount: number
  exitCount: number
  errors: number
}

interface TopPageRow extends TopPageMetric {
  route: RouteDisplayInfo
}

export function TopPagesTable({ pages }: TopPagesTableProps) {
  const { t } = useTranslation()
  const rows = pages.map((page) => ({
    ...page,
    route: resolveRouteDisplayInfo(page.routePath, t),
  }))

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.analytics.tables.pages.title")}</CardTitle>
        <CardDescription>
          {t("admin.analytics.tables.pages.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[280px] overflow-hidden">
          <DataTable<TopPageRow, Record<string, never>>
            columns={buildColumns(t)}
            fetchData={async () => ({ items: rows, total: rows.length })}
            getRowId={(row) => row.routePath}
            height="100%"
            initialPageSize={8}
            initialQuery={{}}
            query={{ builtInFields: [], fields: [], audit: false }}
            bulkDelete={false}
            bulkUpdate={false}
            stickyColumns={{ left: 1 }}
            compactColumns
            compactRows
            fillWidth={false}
          />
        </div>
      </CardContent>
    </Card>
  )
}

function buildColumns(
  t: ReturnType<typeof useTranslation>["t"]
): DataTableColumn<TopPageRow>[] {
  return [
    {
      key: "pageLabel",
      header: t("admin.analytics.tables.pages.columns.pageLabel"),
      width: 260,
      renderCell: (row) => (
        <div className="min-w-0">
          <p className="font-medium">{row.route.label}</p>
          <p className="mt-1 line-clamp-2 text-xs text-(--app-muted-text)">
            {row.route.description}
          </p>
        </div>
      ),
    },
    {
      key: "routePath",
      header: t("admin.analytics.tables.pages.columns.routePath"),
      width: 240,
      renderCell: (row) => (
        <span className="font-mono text-xs">{row.route.rawPath}</span>
      ),
    },
    {
      key: "pv",
      header: t("admin.analytics.tables.common.pv"),
      width: 90,
      renderCell: (row) => formatInteger(row.pv),
    },
    {
      key: "uv",
      header: t("admin.analytics.tables.common.uv"),
      width: 90,
      renderCell: (row) => formatInteger(row.uv),
    },
    {
      key: "sessions",
      header: t("admin.analytics.tables.common.sessions"),
      width: 110,
      renderCell: (row) => formatInteger(row.sessions),
    },
    {
      key: "avgSessionDurationSec",
      header: t("admin.analytics.tables.pages.columns.avgDuration"),
      width: 120,
      renderCell: (row) => formatDuration(row.avgSessionDurationSec),
    },
    {
      key: "entryCount",
      header: t("admin.analytics.tables.pages.columns.entryCount"),
      width: 100,
      renderCell: (row) => formatInteger(row.entryCount),
    },
    {
      key: "exitCount",
      header: t("admin.analytics.tables.pages.columns.exitCount"),
      width: 100,
      renderCell: (row) => formatInteger(row.exitCount),
    },
    {
      key: "errors",
      header: t("admin.analytics.tables.pages.columns.errors"),
      width: 100,
      renderCell: (row) => formatInteger(row.errors),
    },
  ]
}
