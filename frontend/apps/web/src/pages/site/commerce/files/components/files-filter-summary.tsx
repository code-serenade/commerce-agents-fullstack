import type { TrainingFilesFilterSummary } from "../domain/logic"

interface FilesFilterSummaryProps {
  summary: TrainingFilesFilterSummary
  t: (key: string, options?: Record<string, unknown>) => string
}

export function FilesFilterSummary({ summary, t }: FilesFilterSummaryProps) {
  if (summary.total === 0) return null

  return (
    <div className="min-w-0 text-sm text-muted-foreground">
      <span className="whitespace-nowrap">
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
