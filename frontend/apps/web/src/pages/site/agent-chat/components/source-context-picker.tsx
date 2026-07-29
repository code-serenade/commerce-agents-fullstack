import { Info, Upload } from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { Button, SearchInput } from "@workspace/ui-components"
import type {
  AgentChatSourceContextSelection,
  AgentChatSourceLibraryItem,
} from "../domain/agent-chat-context-selection"
import { SourceContextButton } from "./source-context-current-source-button"
import { SourceContextLibrarySection } from "./source-context-library-section"
import { SourceContextSelectionSummary } from "./source-context-selection-summary"
import {
  ComposerPanelEmptyState,
  ComposerPanelFrame,
  ComposerPanelSection,
  composerPanelActionButtonClassName,
  composerPanelListClassName,
  composerPanelTinyButtonClassName,
} from "./composer-panel-layout"

const CURRENT_SOURCE_PAGE_SIZE = 6

export function SourceContextPicker({
  availableSources,
  disabled,
  isLoadingSourceLibrary = false,
  sourceLibrary = [],
  sourceLibraryError = null,
  onAttachExistingSource,
  onDeleteSelectedSources,
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
      <ComposerPanelFrame
        title={t("agentChat.composer.sourceContext.panelTitle")}
        description={t("agentChat.composer.sourceContext.panelHint")}
        actions={
          <>
            <Button
              type="button"
              disabled={disabled || !onUploadClick}
              onClick={onUploadClick}
              classNameMode="replace"
              className={composerPanelActionButtonClassName}
            >
              <Upload className="size-3.5" />
              {t("agentChat.composer.sourceContext.upload")}
            </Button>
          </>
        }
      >
        <div className="grid gap-0">
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
          <ComposerPanelSection
            title={t("agentChat.composer.sourceContext.currentSection")}
            description={t(
              "agentChat.composer.sourceContext.currentSectionHint"
            )}
          >
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
            <div className={composerPanelListClassName}>
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
                <ComposerPanelEmptyState>
                  {t("agentChat.composer.sourceContext.currentNoSearchResult")}
                </ComposerPanelEmptyState>
              ) : (
                <ComposerPanelEmptyState icon={<Info className="size-4" />}>
                  {t("agentChat.composer.sourceContext.empty")}
                </ComposerPanelEmptyState>
              )}
            </div>
            {hiddenCurrentCount > 0 ? (
              <div className="flex justify-center">
                <Button
                  type="button"
                  disabled={disabled}
                  onClick={() =>
                    setVisibleCurrentCount(
                      (current) => current + CURRENT_SOURCE_PAGE_SIZE
                    )
                  }
                  classNameMode="replace"
                  className={composerPanelTinyButtonClassName}
                >
                  {t("agentChat.composer.sourceContext.showMoreCurrent", {
                    count: Math.min(
                      hiddenCurrentCount,
                      CURRENT_SOURCE_PAGE_SIZE
                    ),
                  })}
                </Button>
              </div>
            ) : null}
          </ComposerPanelSection>
          <SourceContextLibrarySection
            disabled={disabled}
            isLoading={isLoadingSourceLibrary}
            onAttachExistingSource={onAttachExistingSource}
            sourceLibrary={sourceLibrary}
            sourceLibraryError={sourceLibraryError}
          />
        </div>
      </ComposerPanelFrame>
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
