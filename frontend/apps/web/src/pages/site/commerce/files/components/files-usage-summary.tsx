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
    <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
      {SUMMARY_ITEMS.map((item) => {
        const Icon = item.icon
        return (
          <div
            key={item.key}
            className="flex h-24 min-w-0 items-center gap-2 rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) px-3 shadow-(--document-card-shadow) backdrop-blur-md"
          >
            <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
              <Icon className="size-4" />
            </span>
            <span className="min-w-0">
              <span className="block text-[11px] text-muted-foreground">
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
