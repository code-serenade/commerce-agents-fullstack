import { useTranslation } from "react-i18next"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components/stable/card"
import { Progress } from "@workspace/ui-components/stable/progress"
import { riskItems } from "../dashboard-data"

export function RiskMonitor() {
  const { t } = useTranslation()

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.riskMonitor.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.riskMonitor.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-5">
          {riskItems.map((item) => (
            <div key={item.labelKey} className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span>{t(item.labelKey)}</span>
                <span>{item.value}%</span>
              </div>
              <Progress value={item.value} />
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
