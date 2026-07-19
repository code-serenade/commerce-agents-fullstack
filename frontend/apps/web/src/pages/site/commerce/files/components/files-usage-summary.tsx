import type { TrainingFilesUsageSummary } from "../domain/logic"

interface FilesUsageSummaryProps {
  summary: TrainingFilesUsageSummary
  t: (key: string, options?: Record<string, unknown>) => string
}

const SUMMARY_ITEMS = [
  { key: "total" },
  { key: "readable" },
] as const

export function FilesUsageSummary({ summary, t }: FilesUsageSummaryProps) {
  return (
    <div className="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-1 text-sm">
      {SUMMARY_ITEMS.map((item, index) => (
        <div key={item.key} className="flex min-w-0 items-center gap-2">
          {index > 0 ? (
            <span className="size-1 rounded-full bg-border" aria-hidden="true" />
          ) : null}
          <span className="text-muted-foreground">
            {t(`commerce.files.summary.${item.key}.label`)}
          </span>
          <span className="font-semibold text-foreground">
            {t(`commerce.files.summary.${item.key}.value`, {
              count: summary[item.key],
            })}
          </span>
        </div>
      ))}
    </div>
  )
}
