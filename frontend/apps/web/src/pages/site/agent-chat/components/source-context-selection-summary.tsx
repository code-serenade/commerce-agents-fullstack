import { Trash2 } from "lucide-react"
import { SourceContextButton } from "./source-context-current-source-button"
import type { AgentChatSourceContextSelection } from "../domain/agent-chat-context-selection"
import {
  sourceReadinessKeys,
  summarizeSelectedSourceReadiness,
  type SourceReadinessKey,
} from "../domain/agent-chat-source-readiness"
import {
  composerPanelSelectedAreaClassName,
  composerPanelTinyButtonClassName,
} from "./composer-panel-layout"

interface SourceContextSelectionSummaryProps {
  availableCount: number
  disabled: boolean
  onClearSourceContext: () => void
  onDeleteSelectedSources?: () => void
  onToggleSourceContext?: (source: AgentChatSourceContextSelection) => void
  readableCount: number
  selectedSources: AgentChatSourceContextSelection[]
  t: (key: string, options?: Record<string, unknown>) => string
}

export function SourceContextSelectionSummary({
  availableCount,
  disabled,
  onClearSourceContext,
  onDeleteSelectedSources,
  onToggleSourceContext,
  readableCount,
  selectedSources,
  t,
}: SourceContextSelectionSummaryProps) {
  const selectedNames = selectedSources
    .slice(0, 3)
    .map((source) => source.title)
    .join(" / ")
  const hiddenCount = Math.max(selectedSources.length - 3, 0)
  const selectedReadiness = summarizeSelectedSourceReadiness(selectedSources)
  const selectedReadinessItems = sourceReadinessKeys.filter(
    (key) => selectedReadiness[key] > 0
  )

  return (
    <div className="mb-3 grid min-w-0 gap-2">
      <div className="flex min-w-0 flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="text-xs font-medium text-foreground">
            {selectedSources.length
              ? t("agentChat.composer.sourceContext.selectedCount", {
                  count: selectedSources.length,
                })
              : t("agentChat.composer.sourceContext.noneSelected")}
          </p>
          <p className="mt-0.5 text-[11px] text-muted-foreground">
            {selectedSources.length
              ? t("agentChat.composer.sourceContext.selectedHint")
              : t("agentChat.composer.sourceContext.manualHint")}
          </p>
          {selectedSources.length ? (
            <p className="mt-1 max-w-xl truncate text-[11px] text-foreground">
              {t("agentChat.composer.sourceContext.selectedPreview", {
                hiddenText: hiddenCount
                  ? t("agentChat.composer.sourceContext.selectedPreviewHidden", {
                      count: hiddenCount,
                    })
                  : "",
                names: selectedNames,
              })}
            </p>
          ) : null}
          {selectedReadinessItems.length ? (
            <div className="mt-1.5 flex min-w-0 flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
              <span>
                {t("agentChat.composer.sourceContext.selectedReadinessLabel")}
              </span>
              {selectedReadinessItems.map((key) => (
                <span
                  key={key}
                  className={`rounded-full px-1.5 py-0.5 ${sourceReadinessClass(key)}`}
                >
                  {t(
                    `agentChat.composer.sourceContext.selectedReadiness.${key}`,
                    { count: selectedReadiness[key] }
                  )}
                </span>
              ))}
            </div>
          ) : null}
        </div>
        <div className="flex shrink-0 flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
          <span className="rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-2 py-0.5">
            {t("agentChat.composer.sourceContext.availableCount", {
              count: availableCount,
            })}
          </span>
          <span className="rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-2 py-0.5">
            {t("agentChat.composer.sourceContext.readableCount", {
              count: readableCount,
            })}
          </span>
        </div>
      </div>
      {selectedSources.length ? (
        <>
          {onToggleSourceContext ? (
            <div className={composerPanelSelectedAreaClassName}>
              {selectedSources.map((source) => (
                <SourceContextButton
                  key={source.ref}
                  disabled={disabled}
                  onToggleSourceContext={onToggleSourceContext}
                  selected
                  source={source}
                />
              ))}
            </div>
          ) : null}
          <div className="flex justify-end gap-2">
            <button
              type="button"
              disabled={disabled}
              onClick={onClearSourceContext}
              className={composerPanelTinyButtonClassName}
            >
              {t("agentChat.composer.sourceContext.clear")}
            </button>
            <button
              type="button"
              disabled={disabled || !onDeleteSelectedSources}
              onClick={onDeleteSelectedSources}
              className="inline-flex h-7 items-center gap-1 rounded-full border border-destructive/35 bg-(--agent-chat-elevated-strong) px-2.5 text-[11px] text-destructive transition-colors hover:bg-destructive/10 disabled:cursor-not-allowed disabled:opacity-50"
            >
              <Trash2 className="size-3" />
              {t("agentChat.composer.sourceContext.deleteSelected")}
            </button>
          </div>
        </>
      ) : null}
    </div>
  )
}

function sourceReadinessClass(key: SourceReadinessKey) {
  if (key === "readable") {
    return "bg-primary/10 text-primary"
  }
  if (key === "pending") {
    return "bg-(--agent-chat-accent-soft) text-(--agent-chat-accent-foreground)"
  }
  if (key === "failed" || key === "unsupported") {
    return "bg-destructive/10 text-destructive"
  }
  return "bg-muted text-muted-foreground"
}
