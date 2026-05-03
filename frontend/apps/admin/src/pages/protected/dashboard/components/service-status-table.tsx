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
import { serviceStatus } from "../dashboard-data"

export function ServiceStatusTable() {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.servicesPanel.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.servicesPanel.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>
                {t("admin.dashboard.servicesPanel.columns.service")}
              </TableHead>
              <TableHead>
                {t("admin.dashboard.servicesPanel.columns.status")}
              </TableHead>
              <TableHead>
                {t("admin.dashboard.servicesPanel.columns.metric")}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {serviceStatus.map((item) => (
              <TableRow key={item.serviceKey}>
                <TableCell>{t(item.serviceKey)}</TableCell>
                <TableCell>{t(item.statusKey)}</TableCell>
                <TableCell>{item.metric}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
