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
import type { DashboardStatusRow } from "../dashboard-types"

export function DataStatusTable({ rows }: { rows: DashboardStatusRow[] }) {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.dataStatus.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.dataStatus.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("admin.dashboard.dataStatus.columns.name")}</TableHead>
              <TableHead>
                {t("admin.dashboard.dataStatus.columns.status")}
              </TableHead>
              <TableHead>
                {t("admin.dashboard.dataStatus.columns.detail")}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((row) => (
              <TableRow key={row.name}>
                <TableCell>
                  <span className="font-medium">{row.name}</span>
                </TableCell>
                <TableCell>{row.status}</TableCell>
                <TableCell>
                  <span className="text-(--app-muted-text)">{row.detail}</span>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
