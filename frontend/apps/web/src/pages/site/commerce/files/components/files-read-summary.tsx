import { readTrainingFileSummary } from "../domain/read-summary"
import type { TrainingFileItem } from "../types"

interface FilesReadSummaryProps {
  file: TrainingFileItem
  t: (key: string, options?: Record<string, unknown>) => string
}

export function FilesReadSummary({ file, t }: FilesReadSummaryProps) {
  const summary = readTrainingFileSummary(file)
  const detailItems = [
    summary.textChars
      ? t("commerce.files.readSummary.textChars", {
          count: summary.textChars,
        })
      : null,
    summary.parser
      ? t("commerce.files.readSummary.parser", {
          parser: summary.parser,
        })
      : null,
    summary.contentKind
      ? t("commerce.files.readSummary.contentKind", {
          kind: summary.contentKind,
        })
      : null,
    summary.truncated ? t("commerce.files.readSummary.truncated") : null,
  ].filter(Boolean)

  if (!summary.summary && !summary.failureReason && detailItems.length === 0) {
    return null
  }

  return (
    <div className="mt-1 grid max-w-md gap-0.5 text-xs text-muted-foreground">
      {summary.summary ? (
        <span className="truncate">
          {t("commerce.files.readSummary.summary", {
            summary: summary.summary,
          })}
        </span>
      ) : null}
      {summary.failureReason ? (
        <span className="truncate text-destructive">
          {t("commerce.files.readSummary.failure", {
            reason: summary.failureReason,
          })}
        </span>
      ) : null}
      {detailItems.length ? (
        <span className="truncate">{detailItems.join(" / ")}</span>
      ) : null}
    </div>
  )
}
