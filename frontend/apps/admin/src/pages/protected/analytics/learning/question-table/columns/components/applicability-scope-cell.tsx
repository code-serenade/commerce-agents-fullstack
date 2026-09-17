import type { TFunction } from "i18next"
import { Badge } from "@workspace/ui-components"
import type { LearningQuestionMetricRow } from "../../../types"

export function LearningQuestionApplicabilityScopeCell({
  row,
  t,
}: {
  row: LearningQuestionMetricRow
  t: TFunction
}) {
  const items = [
    {
      key: "jurisdiction",
      label: t("admin.analytics.learning.questionTable.scopeLabels.jurisdiction"),
      value: t(
        `admin.analytics.learning.questionTable.jurisdictions.${row.jurisdiction_code}`
      ),
    },
    {
      key: "platform",
      label: t("admin.analytics.learning.questionTable.scopeLabels.platform"),
      value: t(
        `admin.analytics.learning.questionTable.platforms.${row.platform_code}`
      ),
    },
    {
      key: "businessStage",
      label: t("admin.analytics.learning.questionTable.scopeLabels.businessStage"),
      value: t(
        `admin.analytics.learning.questionTable.businessStages.${row.business_stage_code}`
      ),
    },
  ]

  return (
    <div className="grid min-w-48 gap-1.5">
      {items.map((item) => (
        <div key={item.key} className="flex min-w-0 items-start gap-2">
          <span className="w-10 shrink-0 pt-0.5 text-[11px]/4 text-muted-foreground">
            {item.label}
          </span>
          <span
            className="min-w-0 max-w-40 *:data-[slot=badge]:max-w-full *:data-[slot=badge]:text-ellipsis"
            title={item.value}
          >
            <Badge variant="outline">{item.value}</Badge>
          </span>
        </div>
      ))}
    </div>
  )
}
