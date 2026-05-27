import { CheckCircle2, Filter, MinusCircle } from "lucide-react"
import { Button } from "@workspace/ui-components"
import type { TrainingFilesTurnSelectionSummary } from "../domain/turn-selection-summary"

interface FilesTurnSelectionSummaryProps {
  canClearSelectedForTurn: boolean
  onClearSelectedForTurn: () => void
  onShowSelectedForTurn: () => void
  summary: TrainingFilesTurnSelectionSummary
  t: (key: string, options?: Record<string, unknown>) => string
}

export function FilesTurnSelectionSummary({
  canClearSelectedForTurn,
  onClearSelectedForTurn,
  onShowSelectedForTurn,
  summary,
  t,
}: FilesTurnSelectionSummaryProps) {
  const hasSelectedFiles = summary.total > 0
  const selectedNames = summary.selectedNames.join(" / ")

  return (
    <div className="flex min-w-0 flex-col gap-2 rounded-lg border border-(--app-border) bg-muted/40 px-3 py-2 sm:flex-row sm:items-center sm:justify-between">
      <div className="flex min-w-0 items-start gap-2">
        <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
        <div className="min-w-0">
          <p className="text-sm font-medium text-foreground">
            {t("commerce.files.turnSelection.title", {
              count: summary.total,
            })}
          </p>
          <p className="truncate text-xs text-muted-foreground">
            {hasSelectedFiles
              ? t("commerce.files.turnSelection.files", {
                  hidden: summary.hiddenNames,
                  names: selectedNames,
                })
              : t("commerce.files.turnSelection.empty")}
          </p>
        </div>
      </div>
      {hasSelectedFiles ? (
        <div className="flex shrink-0 flex-wrap items-center gap-2">
          <Button size="sm" variant="outline" onClick={onShowSelectedForTurn}>
            <Filter className="size-4" />
            {t("commerce.files.turnSelection.action")}
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={!canClearSelectedForTurn}
            onClick={onClearSelectedForTurn}
          >
            <MinusCircle className="size-4" />
            {t("commerce.files.turnSelection.clearAction")}
          </Button>
        </div>
      ) : null}
    </div>
  )
}
