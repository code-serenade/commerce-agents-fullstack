import { Check, FileText, Info } from "lucide-react"
import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import type { AgentChatSourceContextSelection } from "../domain/agent-chat-context-selection"

export function SourceContextPicker({
  availableSources,
  disabled,
  onClearSourceContext,
  onToggleSourceContext,
  selectedSources,
}: {
  availableSources: AgentChatSourceContextSelection[]
  disabled: boolean
  onClearSourceContext: () => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  selectedSources: AgentChatSourceContextSelection[]
}) {
  const { t } = useTranslation()
  const readableCount = useMemo(
    () => availableSources.filter((source) => source.readable).length,
    [availableSources]
  )

  return (
    <>
      <div className="mb-3 grid gap-2">
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
          </div>
          <div className="flex shrink-0 flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
            <span className="rounded-full border border-border bg-background px-2 py-0.5">
              {t("agentChat.composer.sourceContext.availableCount", {
                count: availableSources.length,
              })}
            </span>
            <span className="rounded-full border border-border bg-background px-2 py-0.5">
              {t("agentChat.composer.sourceContext.readableCount", {
                count: readableCount,
              })}
            </span>
          </div>
        </div>
        {selectedSources.length ? (
          <div className="flex justify-end">
            <button
              type="button"
              disabled={disabled}
              onClick={onClearSourceContext}
              className="rounded-full border border-border bg-background px-2.5 py-1 text-[11px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            >
              {t("agentChat.composer.sourceContext.clear")}
            </button>
          </div>
        ) : null}
      </div>
      <div className="grid max-h-72 min-w-0 gap-2 overflow-y-auto pr-1 sm:grid-cols-2">
        {availableSources.length ? (
          availableSources.map((source) => (
            <SourceContextButton
              key={source.ref}
              disabled={disabled}
              onToggleSourceContext={onToggleSourceContext}
              selected={selectedSources.some((item) => item.ref === source.ref)}
              source={source}
            />
          ))
        ) : (
          <div className="col-span-full flex items-start gap-2 rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
            <Info className="mt-0.5 size-4 shrink-0" />
            <span>{t("agentChat.composer.sourceContext.empty")}</span>
          </div>
        )}
      </div>
    </>
  )
}

function SourceContextButton({
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

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => onToggleSourceContext(source)}
      aria-pressed={selected}
      className={`flex min-w-0 items-start gap-2 rounded-lg border px-3 py-2 text-left text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
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
        <span className="mt-1 flex min-w-0 flex-wrap gap-1.5">
          <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
            {source.readable
              ? t("agentChat.composer.sourceContext.readable")
              : t("agentChat.composer.sourceContext.metadataOnly")}
          </span>
          <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
            {t("agentChat.composer.sourceContext.currentConversation")}
          </span>
        </span>
      </span>
    </button>
  )
}
