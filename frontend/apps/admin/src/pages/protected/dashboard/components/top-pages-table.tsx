import { useTranslation } from "react-i18next"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components/stable/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@workspace/ui-components/stable/table"
import { formatInteger } from "../../analytics/access-analytics/format"
import type { DashboardPageRow } from "../dashboard-types"

export function TopPagesTable({ pages }: { pages: DashboardPageRow[] }) {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.topPages.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.topPages.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("admin.dashboard.topPages.columns.page")}</TableHead>
              <TableHead>{t("admin.dashboard.topPages.columns.path")}</TableHead>
              <TableHead>{t("admin.dashboard.topPages.columns.pv")}</TableHead>
              <TableHead>{t("admin.dashboard.topPages.columns.uv")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {pages.map((page) => (
              <TableRow key={page.path}>
                <TableCell>
                  <div className="min-w-0">
                    <p className="font-medium">{page.label}</p>
                    <p className="mt-1 line-clamp-2 text-xs text-(--app-muted-text)">
                      {page.description}
                    </p>
                  </div>
                </TableCell>
                <TableCell>
                  <span className="font-mono text-xs">{page.path}</span>
                </TableCell>
                <TableCell>{formatInteger(page.pv)}</TableCell>
                <TableCell>{formatInteger(page.uv)}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
