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
import { orders } from "../dashboard-data"

export function OrderFlowTable() {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.orders.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.orders.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("admin.dashboard.orders.columns.id")}</TableHead>
              <TableHead>
                {t("admin.dashboard.orders.columns.source")}
              </TableHead>
              <TableHead>
                {t("admin.dashboard.orders.columns.status")}
              </TableHead>
              <TableHead>
                {t("admin.dashboard.orders.columns.amount")}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {orders.map((order) => (
              <TableRow key={order.id}>
                <TableCell>{order.id}</TableCell>
                <TableCell>{t(order.sourceKey)}</TableCell>
                <TableCell>{t(order.statusKey)}</TableCell>
                <TableCell>{order.amount}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
