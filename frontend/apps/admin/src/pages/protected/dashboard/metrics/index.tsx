import { useTranslation } from "react-i18next"
import { Card, CardContent } from "@workspace/ui-components/stable/card"
import { metricCards } from "../dashboard-data"

export function DashboardMetrics() {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      {metricCards.map((card) => (
        <Card key={card.labelKey}>
          <CardContent>
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="text-sm text-(--app-muted-text)">
                  {t(card.labelKey)}
                </p>
                <p className="mt-2 text-3xl font-semibold">{card.value}</p>
                <p className="mt-2 text-sm text-(--success)">{card.delta}</p>
              </div>
              <span className="inline-flex min-h-10 min-w-10 items-center justify-center rounded-xl bg-(--app-active-bg) text-sm font-semibold text-(--app-muted-text)">
                {card.marker}
              </span>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
