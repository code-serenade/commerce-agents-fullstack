import { Filter } from "lucide-react"
import type { TrainingFilesFilterSummary } from "../domain/logic"

interface FilesFilterSummaryProps {
  summary: TrainingFilesFilterSummary
  t: (key: string, options?: Record<string, unknown>) => string
}

export function FilesFilterSummary({ summary, t }: FilesFilterSummaryProps) {
  if (summary.total === 0) return null

  return (
    <div className="flex min-w-0 items-center gap-2 rounded-lg border border-(--app-border) bg-muted/40 px-3 py-2 text-sm text-muted-foreground">
      <Filter className="size-4 shrink-0" />
      <span className="truncate">
        {t(
          summary.active
            ? "commerce.files.filterSummary.filtered"
            : "commerce.files.filterSummary.all",
          {
            filtered: summary.filtered,
            total: summary.total,
          }
        )}
      </span>
    </div>
  )
}
