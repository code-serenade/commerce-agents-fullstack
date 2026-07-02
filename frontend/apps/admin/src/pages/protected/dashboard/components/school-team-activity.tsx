import { useTranslation } from "react-i18next"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components/stable/card"
import { Progress } from "@workspace/ui-components/stable/progress"
import { formatInteger } from "../../analytics/access-analytics/format"
import type { DashboardSchoolTeamRow } from "../dashboard-types"

export function SchoolTeamActivity({
  rows,
}: {
  rows: DashboardSchoolTeamRow[]
}) {
  const { t } = useTranslation()
  const maxPv = Math.max(...rows.map((row) => row.pv), 1)

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("admin.dashboard.schoolTeams.title")}</CardTitle>
        <CardDescription>
          {t("admin.dashboard.schoolTeams.description")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {rows.length > 0 ? (
            rows.map((row) => (
              <div key={row.label} className="space-y-2">
                <div className="flex items-start justify-between gap-3 text-sm">
                  <div className="min-w-0">
                    <p className="truncate font-medium">{row.label}</p>
                    <p className="mt-0.5 text-xs text-(--app-muted-text)">
                      {t("admin.dashboard.schoolTeams.rowDetail", {
                        accounts: formatInteger(row.activeAccounts),
                        questions: formatInteger(row.agentQuestions),
                      })}
                    </p>
                  </div>
                  <span className="shrink-0 font-semibold">
                    {formatInteger(row.pv)}
                  </span>
                </div>
                <Progress value={(row.pv / maxPv) * 100} />
              </div>
            ))
          ) : (
            <p className="text-sm text-(--app-muted-text)">
              {t("admin.dashboard.schoolTeams.empty")}
            </p>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
