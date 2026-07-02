import { useTranslation } from "react-i18next"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components/stable/card"
import type { AnalyticsDateRange } from "../../analytics/access-analytics/types"

export function DashboardHeading({
  isLoading,
  range,
}: {
  isLoading: boolean
  range: AnalyticsDateRange
}) {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.heading.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.heading.description", {
            from: range.from,
            status: isLoading
              ? t("admin.dashboard.heading.loading")
              : t("admin.dashboard.heading.ready"),
            to: range.to,
          })}
        </CardDescription>
      </CardHeader>
    </Card>
  )
}
