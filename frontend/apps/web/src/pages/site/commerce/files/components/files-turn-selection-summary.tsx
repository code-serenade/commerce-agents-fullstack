import { CheckCircle2, Filter, X } from "lucide-react"
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
  return (
    <div className="flex min-w-0 flex-wrap items-center gap-2 sm:justify-end">
      <div className="flex min-w-0 items-center gap-2 text-sm">
        <CheckCircle2
          className={hasSelectedFiles ? "size-4 text-primary" : "size-4 text-muted-foreground"}
        />
        <span className={hasSelectedFiles ? "font-medium text-foreground" : "text-muted-foreground"}>
          {hasSelectedFiles
            ? t("commerce.files.turnSelection.title", { count: summary.total })
            : t("commerce.files.turnSelection.empty")}
        </span>
      </div>
      {hasSelectedFiles ? (
        <div className="flex shrink-0 items-center gap-1">
          <Button size="xs" variant="ghost" onClick={onShowSelectedForTurn}>
            <Filter className="size-4" />
            {t("commerce.files.turnSelection.action")}
          </Button>
          <Button
            size="xs"
            variant="ghost"
            disabled={!canClearSelectedForTurn}
            onClick={onClearSelectedForTurn}
          >
            <X className="size-4" />
            {t("commerce.files.turnSelection.clearAction")}
          </Button>
        </div>
      ) : null}
    </div>
  )
}
