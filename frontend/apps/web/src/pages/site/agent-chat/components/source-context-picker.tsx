import { FolderOpen, Info, Upload } from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { SearchInput } from "@workspace/ui-components"
import type {
  AgentChatSourceContextSelection,
  AgentChatSourceLibraryItem,
} from "../domain/agent-chat-context-selection"
import { SourceContextButton } from "./source-context-current-source-button"
import { SourceContextLibrarySection } from "./source-context-library-section"
import { SourceContextSelectionSummary } from "./source-context-selection-summary"

const CURRENT_SOURCE_PAGE_SIZE = 6

export function SourceContextPicker({
  availableSources,
  disabled,
  isLoadingSourceLibrary = false,
  sourceLibrary = [],
  sourceLibraryError = null,
  onAttachExistingSource,
  onDeleteSelectedSources,
  onManageSources,
  onClearSourceContext,
  onToggleSourceContext,
  onUploadClick,
  selectedSources,
}: {
  availableSources: AgentChatSourceContextSelection[]
  disabled: boolean
  isLoadingSourceLibrary?: boolean
  sourceLibrary?: AgentChatSourceLibraryItem[]
  sourceLibraryError?: string | null
  onAttachExistingSource?: (fileId: string) => void
  onDeleteSelectedSources?: () => void
  onManageSources?: () => void
  onClearSourceContext: () => void
  onToggleSourceContext: (source: AgentChatSourceContextSelection) => void
  onUploadClick?: () => void
  selectedSources: AgentChatSourceContextSelection[]
}) {
  const { t } = useTranslation()
  const [currentQuery, setCurrentQuery] = useState("")
  const [visibleCurrentCount, setVisibleCurrentCount] = useState(
    CURRENT_SOURCE_PAGE_SIZE
  )
  const readableCount = useMemo(
    () =>
      availableSources.filter((source) => source.readable || source.visionReady)
        .length,
    [availableSources]
  )
  const filteredCurrentSources = useMemo(
    () => filterCurrentSources(availableSources, currentQuery),
    [availableSources, currentQuery]
  )
  const visibleCurrentSources = filteredCurrentSources.slice(
    0,
    visibleCurrentCount
  )
  const hiddenCurrentCount = Math.max(
    filteredCurrentSources.length - visibleCurrentSources.length,
    0
  )
  const confirmDeleteSelectedSources = () => {
    if (!onDeleteSelectedSources) return
    const names = selectedSources
      .slice(0, 3)
      .map((source) => source.title)
      .join(" / ")
    const hidden = Math.max(selectedSources.length - 3, 0)
    const confirmed = window.confirm(
      t("agentChat.composer.sourceContext.deleteConfirm", {
        count: selectedSources.length,
        hiddenText: hidden
          ? t("agentChat.composer.sourceContext.deleteConfirmHidden", {
              count: hidden,
            })
          : "",
        names,
      })
    )
    if (confirmed) onDeleteSelectedSources()
  }

  return (
    <>
      <div className="grid gap-3">
        <div className="flex min-w-0 flex-wrap items-center justify-between gap-2 rounded-lg border border-border bg-muted/35 p-2">
          <div className="min-w-0">
            <p className="text-xs/5 font-semibold text-foreground">
              {t("agentChat.composer.sourceContext.panelTitle")}
            </p>
            <p className="mt-0.5 text-[11px]/4 text-muted-foreground">
              {t("agentChat.composer.sourceContext.panelHint")}
            </p>
          </div>
          <div className="flex shrink-0 flex-wrap items-center gap-1.5">
            <button
              type="button"
              disabled={disabled || !onUploadClick}
              onClick={onUploadClick}
              className="inline-flex h-8 items-center gap-1.5 rounded-md bg-foreground px-3 text-xs font-medium text-background transition-colors hover:bg-foreground/90 disabled:cursor-not-allowed disabled:opacity-50"
            >
              <Upload className="size-3.5" />
              {t("agentChat.composer.sourceContext.upload")}
            </button>
            <button
              type="button"
              disabled={!onManageSources}
              onClick={onManageSources}
              className="inline-flex h-8 items-center gap-1.5 rounded-md border border-border bg-background px-3 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            >
              <FolderOpen className="size-3.5" />
              {t("agentChat.composer.sourceContext.manageLibrary")}
            </button>
          </div>
        </div>
        <SourceContextSelectionSummary
          availableCount={availableSources.length}
          disabled={disabled}
          onClearSourceContext={onClearSourceContext}
          onDeleteSelectedSources={
            onDeleteSelectedSources ? confirmDeleteSelectedSources : undefined
          }
          onToggleSourceContext={onToggleSourceContext}
          readableCount={readableCount}
          selectedSources={selectedSources}
          t={t}
        />
        <div className="grid min-w-0 gap-2">
          <div className="flex min-w-0 items-center justify-between gap-2">
            <p className="text-[11px] font-semibold text-foreground">
              {t("agentChat.composer.sourceContext.currentSection")}
            </p>
            <p className="text-[11px] text-muted-foreground">
              {t("agentChat.composer.sourceContext.currentSectionHint")}
            </p>
          </div>
          {availableSources.length ? (
            <SearchInput
              disabled={disabled}
              onValueChange={(value) => {
                setCurrentQuery(value)
                setVisibleCurrentCount(CURRENT_SOURCE_PAGE_SIZE)
              }}
              placeholder={t(
                "agentChat.composer.sourceContext.currentSearch"
              )}
              value={currentQuery}
            />
          ) : null}
          <div className="grid min-w-0 gap-2 sm:grid-cols-2">
            {availableSources.length && filteredCurrentSources.length ? (
              visibleCurrentSources.map((source) => (
                <SourceContextButton
                  key={source.ref}
                  disabled={disabled}
                  onToggleSourceContext={onToggleSourceContext}
                  selected={selectedSources.some(
                    (item) => item.ref === source.ref
                  )}
                  source={source}
                />
              ))
            ) : availableSources.length ? (
              <div className="col-span-full rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
                {t("agentChat.composer.sourceContext.currentNoSearchResult")}
              </div>
            ) : (
              <div className="col-span-full flex items-start gap-2 rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
                <Info className="mt-0.5 size-4 shrink-0" />
                <span>{t("agentChat.composer.sourceContext.empty")}</span>
              </div>
            )}
          </div>
          {hiddenCurrentCount > 0 ? (
            <div className="flex justify-center">
              <button
                type="button"
                disabled={disabled}
                onClick={() =>
                  setVisibleCurrentCount(
                    (current) => current + CURRENT_SOURCE_PAGE_SIZE
                  )
                }
                className="rounded-full border border-border bg-background px-3 py-1 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
              >
                {t("agentChat.composer.sourceContext.showMoreCurrent", {
                  count: Math.min(hiddenCurrentCount, CURRENT_SOURCE_PAGE_SIZE),
                })}
              </button>
            </div>
          ) : null}
        </div>
        <SourceContextLibrarySection
          disabled={disabled}
          isLoading={isLoadingSourceLibrary}
          onAttachExistingSource={onAttachExistingSource}
          onManageSources={onManageSources}
          sourceLibrary={sourceLibrary}
          sourceLibraryError={sourceLibraryError}
        />
      </div>
    </>
  )
}

function filterCurrentSources(
  sources: AgentChatSourceContextSelection[],
  query: string
) {
  const normalizedQuery = query.trim().toLowerCase()
  if (!normalizedQuery) return sources

  return sources.filter((source) =>
    [
      source.title,
      source.status,
      source.readable ? "readable" : "",
      source.visionReady ? "image vision" : "",
    ]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(normalizedQuery))
  )
}
