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
import type { DashboardUsageAlert } from "../dashboard-types"

export function UsageAlertsTable({ rows }: { rows: DashboardUsageAlert[] }) {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.usageAlerts.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.usageAlerts.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>
                {t("admin.dashboard.usageAlerts.columns.account")}
              </TableHead>
              <TableHead>
                {t("admin.dashboard.usageAlerts.columns.schoolTeam")}
              </TableHead>
              <TableHead>{t("admin.dashboard.usageAlerts.columns.used")}</TableHead>
              <TableHead>
                {t("admin.dashboard.usageAlerts.columns.remaining")}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((row) => (
              <TableRow key={row.account}>
                <TableCell>
                  <span className="font-medium">{row.account}</span>
                </TableCell>
                <TableCell>{row.schoolTeam}</TableCell>
                <TableCell>{formatInteger(row.used)}</TableCell>
                <TableCell>{formatInteger(row.remaining)}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
