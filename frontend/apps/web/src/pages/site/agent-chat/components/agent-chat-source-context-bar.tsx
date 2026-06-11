import { ChevronDown, Files, TriangleAlert } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type {
  AgentChatSourceContextSelection,
  AgentChatSourceLibraryItem,
} from "../domain/agent-chat-context-selection"
import {
  sourceReadinessKeys,
  summarizeSelectedSourceReadiness,
} from "../domain/agent-chat-source-readiness"
import { SourceContextPicker } from "./source-context-picker"

export interface AgentChatSourceContextBarProps {
  availableSources: AgentChatSourceContextSelection[]
  disabled: boolean
  isLoadingSourceLibrary?: boolean
  isOpen?: boolean
  onAttachExistingSource?: (fileId: string) => void
  onDeleteSelectedSources?: () => void
  onManageSources?: () => void
  onClearSourceContext: () => void
  onOpenChange?: (open: boolean) => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  onUploadClick?: () => void
  selectedSources: AgentChatSourceContextSelection[]
  sourceLibrary?: AgentChatSourceLibraryItem[]
  sourceLibraryError?: string | null
  variant?: "compact" | "panel"
}

export function AgentChatSourceContextBar({
  availableSources,
  disabled,
  isLoadingSourceLibrary = false,
  isOpen,
  onAttachExistingSource,
  onDeleteSelectedSources,
  onManageSources,
  onClearSourceContext,
  onOpenChange,
  onToggleSourceContext,
  onUploadClick,
  selectedSources,
  sourceLibrary = [],
  sourceLibraryError = null,
  variant = "compact",
}: AgentChatSourceContextBarProps) {
  const { t } = useTranslation()
  const [internalOpen, setInternalOpen] = useState(false)
  const open = isOpen ?? internalOpen
  const sourceNames = selectedSources.map((source) => source.title)
  const selectedReadiness = summarizeSelectedSourceReadiness(selectedSources)
  const selectedReadinessItems = sourceReadinessKeys.filter(
    (key) => selectedReadiness[key] > 0
  )
  const unreadableSelectedCount =
    selectedReadiness.pending +
    selectedReadiness.failed +
    selectedReadiness.unsupported +
    selectedReadiness.metadataOnly
  const selectedReadinessSummary = selectedReadinessItems
    .map((key) =>
      t(`agentChat.composer.sourceContext.selectedReadiness.${key}`, {
        count: selectedReadiness[key],
      })
    )
    .join(" / ")
  const selectedSourceSummary =
    selectedSources.length > 0
      ? t("agentChat.composer.sourceContext.selectedCount", {
          count: selectedSources.length,
        })
      : t("agentChat.composer.sourceContext.noneSelected")
  const selectedReadinessTitle = selectedSources.length
    ? t("agentChat.composer.sourceContext.compactReadinessTitle", {
        status: selectedReadinessSummary,
      })
    : ""
  const title = selectedSources.length
    ? `${t("agentChat.composer.sourceContext.compactTitle")} - ${sourceNames.join(" / ")} - ${selectedReadinessTitle}`
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
        isLoadingSourceLibrary={isLoadingSourceLibrary}
        sourceLibrary={sourceLibrary}
        sourceLibraryError={sourceLibraryError}
        onAttachExistingSource={onAttachExistingSource}
        onDeleteSelectedSources={onDeleteSelectedSources}
        onManageSources={onManageSources}
        onClearSourceContext={onClearSourceContext}
        onToggleSourceContext={onToggleSourceContext}
        onUploadClick={onUploadClick}
        selectedSources={selectedSources}
      />
    )
  }

  return (
    <div className="relative shrink-0" data-composer-interactive="true">
      <button
        type="button"
        onClick={toggleOpen}
        className={`flex h-7 max-w-44 shrink-0 items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium transition-colors sm:max-w-48 ${
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
        {unreadableSelectedCount ? (
          <span className="inline-flex h-4 min-w-4 shrink-0 items-center justify-center gap-0.5 rounded-full bg-amber-500/15 px-1 text-[10px] text-amber-700 dark:text-amber-200">
            <TriangleAlert className="size-3" />
            <span>{unreadableSelectedCount}</span>
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
