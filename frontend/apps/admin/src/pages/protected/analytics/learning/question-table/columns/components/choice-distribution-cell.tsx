import { useTranslation } from "react-i18next"
import { Progress } from "@workspace/ui-components"
import type { LearningQuestionChoiceMetricRow } from "../../../types"

function optionLabel(index: number) {
  return String.fromCharCode("A".charCodeAt(0) + index)
}

function formatRate(value: number) {
  return Math.round(value * 100)
}

export function LearningChoiceDistributionCell({
  choices,
  unmappedChoiceCount,
}: {
  choices: LearningQuestionChoiceMetricRow[]
  unmappedChoiceCount: number
}) {
  const { t } = useTranslation()
  const mappedChoiceCount = choices.reduce(
    (total, choice) => total + choice.selected_count,
    0
  )

  if (!choices.length) {
    return (
      <span className="text-xs text-muted-foreground">
        {t("admin.analytics.learning.questionTable.noChoiceResponses")}
      </span>
    )
  }

  return (
    <div className="min-w-72 space-y-2 py-1">
      {choices.map((choice, index) => {
        const label = optionLabel(index)
        const rate = formatRate(choice.selected_rate)
        return (
          <div
            key={choice.choice_id}
            aria-label={`${label}. ${choice.choice_text}. ${choice.selected_count}. ${rate}%`}
            className="space-y-1"
          >
            <div className="flex min-w-0 items-center gap-2 text-xs/4">
              <span className="flex size-5 shrink-0 items-center justify-center rounded border border-border bg-background font-medium text-foreground">
                {label}
              </span>
              <span
                className="min-w-0 flex-1 truncate text-foreground"
                title={choice.choice_text}
              >
                {choice.choice_text}
              </span>
              <span className="shrink-0 tabular-nums text-muted-foreground">
                {t("admin.analytics.learning.questionTable.choiceSelection", {
                  count: choice.selected_count,
                  rate,
                })}
              </span>
            </div>
            <Progress value={rate} />
          </div>
        )
      })}
      {mappedChoiceCount === 0 ? (
        <p className="text-[11px]/4 text-muted-foreground">
          {t("admin.analytics.learning.questionTable.noChoiceResponses")}
        </p>
      ) : null}
      {unmappedChoiceCount > 0 ? (
        <p className="text-[11px]/4 text-muted-foreground">
          {t("admin.analytics.learning.questionTable.unmappedChoices", {
            count: unmappedChoiceCount,
          })}
        </p>
      ) : null}
    </div>
  )
}
