import { FileText } from "lucide-react"
import { useTranslation } from "react-i18next"
import { getInlineArtifacts } from "../view-model/thread-inline-view-model"
import { InlineArtifactCard } from "./inline-artifact-card"

export function InlineArtifactsPanel({
  artifacts,
  hasRuntime,
  isStreaming,
}: {
  artifacts: ReturnType<typeof getInlineArtifacts>
  hasRuntime: boolean
  isStreaming: boolean
}) {
  const { t } = useTranslation()

  return (
    <div className="mt-3 rounded-lg border border-border bg-background/80 p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2 text-sm font-medium">
          <FileText className="size-4 shrink-0 text-muted-foreground" />
          <span>{t("agentChat.thread.artifactsTitle")}</span>
          <span className="truncate text-xs font-normal text-muted-foreground">
            {artifacts.filter((artifact) => artifact.state === "done").length}/
            {artifacts.length}
          </span>
        </div>
        <span className="rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
          {hasRuntime
            ? t("agentChat.thread.artifacts.updating")
            : isStreaming
              ? t("agentChat.thread.artifacts.preparing")
              : t("agentChat.thread.artifacts.pending")}
        </span>
      </div>
      <div className="mt-3 grid gap-2">
        {artifacts.map((artifact) => (
          <InlineArtifactCard key={artifact.filename} artifact={artifact} />
        ))}
      </div>
    </div>
  )
}
