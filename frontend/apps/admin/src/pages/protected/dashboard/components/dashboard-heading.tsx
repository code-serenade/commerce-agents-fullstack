import { useTranslation } from "react-i18next"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components/stable/card"

export function DashboardHeading() {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.heading.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.heading.description")}
        </CardDescription>
      </CardHeader>
    </Card>
  )
}
