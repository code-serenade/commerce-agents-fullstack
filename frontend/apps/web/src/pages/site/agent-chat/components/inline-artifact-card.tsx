import { CheckCircle2, FileText, Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  getInlineArtifactStateLabel,
  type InlineArtifact,
} from "../view-model/thread-inline-view-model"

export function InlineArtifactCard({ artifact }: { artifact: InlineArtifact }) {
  const { t } = useTranslation()

  return (
    <div
      className={`flex min-w-0 items-center gap-3 rounded-lg border px-3 py-2 ${
        artifact.state === "done"
          ? "border-emerald-200 bg-emerald-50/70 text-emerald-950 dark:border-emerald-900/60 dark:bg-emerald-950/20 dark:text-emerald-100"
          : artifact.state === "working" || artifact.state === "reworking"
            ? "border-foreground/25 bg-background"
            : artifact.state === "failed"
              ? "border-destructive/30 bg-destructive/10"
              : "border-border bg-card"
      }`}
    >
      <div className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-background text-muted-foreground">
        {artifact.state === "working" || artifact.state === "reworking" ? (
          <Loader2 className="size-4 animate-spin text-emerald-600" />
        ) : artifact.state === "done" ? (
          <CheckCircle2 className="size-4 text-emerald-600" />
        ) : (
          <FileText className="size-4" />
        )}
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 items-center gap-2">
          <p className="truncate text-sm font-medium">{artifact.filename}</p>
          <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
            {artifact.meta}
          </span>
        </div>
        <p className="mt-0.5 truncate text-xs text-muted-foreground">
          {artifact.status}
        </p>
      </div>
      <span className="hidden shrink-0 text-xs text-muted-foreground sm:inline">
        {getInlineArtifactStateLabel(artifact.state, t)}
      </span>
    </div>
  )
}
