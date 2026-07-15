import { CheckCircle2, FileCheck2, FileText, Library } from "lucide-react"
import type { TrainingFilesUsageSummary } from "../domain/logic"

interface FilesUsageSummaryProps {
  summary: TrainingFilesUsageSummary
  t: (key: string, options?: Record<string, unknown>) => string
}

const SUMMARY_ITEMS = [
  {
    icon: Library,
    key: "total",
  },
  {
    icon: FileText,
    key: "readable",
  },
  {
    icon: FileCheck2,
    key: "attached",
  },
  {
    icon: CheckCircle2,
    key: "selectedForTurn",
  },
] as const

export function FilesUsageSummary({ summary, t }: FilesUsageSummaryProps) {
  return (
    <div className="grid overflow-hidden rounded-lg border border-(--app-border) bg-muted/30 sm:grid-cols-2 xl:grid-cols-4">
      {SUMMARY_ITEMS.map((item) => {
        const Icon = item.icon
        return (
          <div
            key={item.key}
            className="flex h-16 min-w-0 items-center gap-3 border-b border-(--app-border) px-3 last:border-b-0 sm:[&:nth-child(odd)]:border-r sm:[&:nth-last-child(-n+2)]:border-b-0 xl:border-r xl:border-b-0 xl:last:border-r-0"
          >
            <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
              <Icon className="size-4" />
            </span>
            <span className="min-w-0">
              <span className="block text-xs text-muted-foreground">
                {t(`commerce.files.summary.${item.key}.label`)}
              </span>
              <span className="block truncate text-sm font-semibold text-foreground">
                {t(`commerce.files.summary.${item.key}.value`, {
                  count: summary[item.key],
                })}
              </span>
            </span>
          </div>
        )
      })}
    </div>
  )
}
