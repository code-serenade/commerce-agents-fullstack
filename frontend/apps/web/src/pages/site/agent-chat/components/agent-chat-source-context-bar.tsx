import { ChevronDown, Files } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AgentChatSourceContextSelection } from "../domain/agent-chat-context-selection"
import { SourceContextPicker } from "./source-context-picker"

export interface AgentChatSourceContextBarProps {
  availableSources: AgentChatSourceContextSelection[]
  disabled: boolean
  isOpen?: boolean
  onClearSourceContext: () => void
  onOpenChange?: (open: boolean) => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  selectedSources: AgentChatSourceContextSelection[]
  variant?: "compact" | "panel"
}

export function AgentChatSourceContextBar({
  availableSources,
  disabled,
  isOpen,
  onClearSourceContext,
  onOpenChange,
  onToggleSourceContext,
  selectedSources,
  variant = "compact",
}: AgentChatSourceContextBarProps) {
  const { t } = useTranslation()
  const [internalOpen, setInternalOpen] = useState(false)
  const open = isOpen ?? internalOpen
  const sourceNames = selectedSources.map((source) => source.title)
  const selectedSourceSummary =
    selectedSources.length > 0
      ? t("agentChat.composer.sourceContext.selectedCount", {
          count: selectedSources.length,
        })
      : t("agentChat.composer.sourceContext.noneSelected")
  const title = selectedSources.length
    ? `${t("agentChat.composer.sourceContext.compactTitle")} - ${sourceNames.join(" / ")}`
    : `${t("agentChat.composer.sourceContext.compactTitle")} - ${selectedSourceSummary}`

  const toggleOpen = () => {
    const nextOpen = !open
    if (isOpen === undefined) setInternalOpen(nextOpen)
    onOpenChange?.(nextOpen)
  }

  if (variant === "panel") {
    return (
      <SourceContextPicker
        availableSources={availableSources}
        disabled={disabled}
        onClearSourceContext={onClearSourceContext}
        onToggleSourceContext={onToggleSourceContext}
        selectedSources={selectedSources}
      />
    )
  }

  if (!selectedSources.length && !availableSources.length) return null

  return (
    <div className="relative shrink-0" data-composer-interactive="true">
      <button
        type="button"
        onClick={toggleOpen}
        className={`flex h-7 max-w-36 shrink-0 items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium transition-colors ${
          open
            ? "border-amber-400/80 bg-amber-50 text-amber-800 dark:bg-amber-500/15 dark:text-amber-100"
            : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
        }`}
        aria-expanded={open}
        aria-label={title}
        title={title}
      >
        <Files className="size-3.5 shrink-0" />
        <span className="truncate">
          {t("agentChat.composer.sourceContext.compactLabel")}
        </span>
        {selectedSources.length ? (
          <span className="flex size-4 shrink-0 items-center justify-center rounded-full bg-current/10 text-[10px]">
            {selectedSources.length}
          </span>
        ) : null}
        <ChevronDown
          className={`size-3.5 shrink-0 transition-transform ${
            open ? "rotate-180" : ""
          }`}
        />
      </button>
    </div>
  )
}
