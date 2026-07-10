import { Check, Eye, FileText } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type { AgentChatSourceContextSelection } from "../domain/agent-chat-context-selection"

export function SourceContextButton({
  disabled,
  onToggleSourceContext,
  selected,
  source,
}: {
  disabled: boolean
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  selected: boolean
  source: AgentChatSourceContextSelection
}) {
  const { t } = useTranslation()
  const readStatus = getSourceReadStatus(source)

  return (
    <Button
      type="button"
      disabled={disabled}
      onClick={() => onToggleSourceContext(source)}
      aria-pressed={selected}
      classNameMode="replace"
      className={`flex min-w-0 items-start gap-2 rounded-md border px-2.5 py-1.5 text-left text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-amber-400/80 bg-amber-50 text-amber-800 dark:bg-amber-500/15 dark:text-amber-100"
          : "border-border bg-background/70 text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={source.title}
    >
      {selected ? (
        <span className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[4px] border border-amber-500 bg-amber-500 text-white">
          <Check className="size-3" />
        </span>
      ) : (
        <span className="mt-0.5 size-4 shrink-0 rounded-[4px] border border-border bg-background shadow-inner" />
      )}
      <span className="min-w-0 flex-1">
        <span className="flex min-w-0 items-center gap-1.5">
          <FileText className="size-3.5 shrink-0" />
          <span className="truncate font-medium text-foreground">
            {source.title}
          </span>
        </span>
        <span className="mt-0.5 flex min-w-0 flex-wrap gap-1">
          <span
            className={`rounded-full px-1.5 py-0.5 text-[10px] ${readStatusClass(readStatus)}`}
          >
            {t(`agentChat.composer.sourceContext.sourceStatus.${readStatus}`)}
          </span>
          {source.visionReady ? (
            <span className="inline-flex items-center gap-1 rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
              <Eye className="size-3" />
              {t("agentChat.composer.sourceContext.vision")}
            </span>
          ) : null}
          <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
            {t("agentChat.composer.sourceContext.currentConversation")}
          </span>
        </span>
      </span>
    </Button>
  )
}

type SourceReadStatus =
  | "failed"
  | "metadataOnly"
  | "pending"
  | "readable"
  | "unsupported"
  | "visionReady"

function getSourceReadStatus(
  source: AgentChatSourceContextSelection
): SourceReadStatus {
  if (source.visionReady) return "visionReady"
  if (source.readable) return "readable"
  if (source.status === "pending") return "pending"
  if (source.status === "failed") return "failed"
  if (source.status === "unsupported") return "unsupported"
  return "metadataOnly"
}

function readStatusClass(status: SourceReadStatus) {
  if (status === "failed" || status === "unsupported") {
    return "bg-destructive/10 text-destructive"
  }
  if (status === "pending") {
    return "bg-amber-500/10 text-amber-700 dark:text-amber-200"
  }
  if (status === "readable" || status === "visionReady") {
    return "bg-emerald-500/10 text-emerald-700 dark:text-emerald-200"
  }
  return "bg-muted text-muted-foreground"
}
