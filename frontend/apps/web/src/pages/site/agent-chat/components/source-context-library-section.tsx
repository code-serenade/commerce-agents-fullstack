import {
  CheckCircle2,
  FilePlus2,
  FileText,
  Loader2,
} from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { SearchInput } from "@workspace/ui-components"
import type { AgentChatSourceLibraryItem } from "../domain/agent-chat-context-selection"
import {
  ComposerPanelEmptyState,
  ComposerPanelSection,
  composerPanelListClassName,
  composerPanelTinyButtonClassName,
} from "./composer-panel-layout"

const LIBRARY_PAGE_SIZE = 6

export function SourceContextLibrarySection({
  disabled,
  isLoading,
  onAttachExistingSource,
  sourceLibrary,
  sourceLibraryError,
}: {
  disabled: boolean
  isLoading: boolean
  onAttachExistingSource?: (fileId: string) => void
  sourceLibrary: AgentChatSourceLibraryItem[]
  sourceLibraryError: string | null
}) {
  const { t } = useTranslation()
  const [query, setQuery] = useState("")
  const [visibleCount, setVisibleCount] = useState(LIBRARY_PAGE_SIZE)
  const filteredLibrary = useMemo(
    () => filterSourceLibrary(sourceLibrary, query),
    [sourceLibrary, query]
  )
  const visibleLibrary = filteredLibrary.slice(0, visibleCount)
  const hiddenCount = Math.max(filteredLibrary.length - visibleLibrary.length, 0)
  const visible =
    isLoading ||
    Boolean(sourceLibraryError) ||
    sourceLibrary.length > 0

  if (!visible) return null

  return (
    <ComposerPanelSection
      className="mt-4 border-t border-(--agent-chat-soft-border) pt-4"
      title={t("agentChat.composer.sourceContext.librarySection")}
      description={t("agentChat.composer.sourceContext.libraryHint")}
    >
      {sourceLibrary.length ? (
        <SearchInput
          disabled={disabled}
          onValueChange={(value) => {
            setQuery(value)
            setVisibleCount(LIBRARY_PAGE_SIZE)
          }}
          placeholder={t("agentChat.composer.sourceContext.librarySearch")}
          value={query}
        />
      ) : null}
      {sourceLibraryError ? (
        <div className="rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-xs text-destructive">
          {sourceLibraryError}
        </div>
      ) : null}
      {isLoading ? (
        <div className="flex items-center gap-2 rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) p-3 text-xs text-muted-foreground">
          <Loader2 className="size-4 animate-spin" />
          <span>{t("agentChat.composer.sourceContext.libraryLoading")}</span>
        </div>
      ) : null}
      {!isLoading && sourceLibrary.length && filteredLibrary.length ? (
        <div className={composerPanelListClassName}>
          {visibleLibrary.map((source) => (
            <SourceLibraryButton
              key={source.id}
              disabled={disabled}
              onAttachExistingSource={onAttachExistingSource}
              source={source}
            />
          ))}
        </div>
      ) : null}
      {!isLoading && sourceLibrary.length && !filteredLibrary.length ? (
        <ComposerPanelEmptyState>
          {t("agentChat.composer.sourceContext.libraryNoSearchResult")}
        </ComposerPanelEmptyState>
      ) : null}
      {!isLoading && hiddenCount > 0 ? (
        <div className="flex justify-center">
          <button
            type="button"
            disabled={disabled}
            onClick={() =>
              setVisibleCount((current) => current + LIBRARY_PAGE_SIZE)
            }
            className={composerPanelTinyButtonClassName}
          >
            {t("agentChat.composer.sourceContext.showMoreLibrary", {
              count: Math.min(hiddenCount, LIBRARY_PAGE_SIZE),
            })}
          </button>
        </div>
      ) : null}
    </ComposerPanelSection>
  )
}

function filterSourceLibrary(
  sourceLibrary: AgentChatSourceLibraryItem[],
  query: string
) {
  const normalizedQuery = query.trim().toLowerCase()
  if (!normalizedQuery) return sourceLibrary

  return sourceLibrary.filter((source) =>
    [
      source.title,
      source.mimeType,
      source.status,
      source.readable ? "readable" : "",
      source.visionReady ? "image vision" : "",
      source.attached ? "attached current conversation" : "",
    ]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(normalizedQuery))
  )
}

function SourceLibraryButton({
  disabled,
  onAttachExistingSource,
  source,
}: {
  disabled: boolean
  onAttachExistingSource?: (fileId: string) => void
  source: AgentChatSourceLibraryItem
}) {
  const { t } = useTranslation()
  const statusKey = source.visionReady
    ? "visionReady"
    : source.status === "readable"
      ? "readable"
      : source.status

  return (
    <button
      type="button"
      disabled={disabled || source.attached || !onAttachExistingSource}
      onClick={() => onAttachExistingSource?.(source.id)}
      className={`flex min-w-0 items-start gap-2 rounded-lg border px-3 py-2 text-left text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-60 ${
        source.attached
          ? "border-primary/45 bg-primary/10 text-primary"
          : "border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-muted-foreground hover:bg-(--agent-chat-accent-soft) hover:text-foreground"
      }`}
      title={source.title}
    >
      <span
        className={`mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[4px] border ${
          source.attached
            ? "border-primary bg-primary text-primary-foreground"
            : "border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-muted-foreground"
        }`}
      >
        {source.attached ? (
          <CheckCircle2 className="size-3" />
        ) : (
          <FilePlus2 className="size-3" />
        )}
      </span>
      <span className="min-w-0 flex-1">
        <span className="flex min-w-0 items-center gap-1.5">
          <FileText className="size-3.5 shrink-0" />
          <span className="truncate font-medium text-foreground">
            {source.title}
          </span>
        </span>
        <span className="mt-1 flex min-w-0 flex-wrap gap-1.5">
          <span className="rounded-full bg-(--agent-chat-subtle-surface) px-1.5 py-0.5 text-[10px] text-muted-foreground">
            {t(`agentChat.composer.sourceContext.libraryStatus.${statusKey}`)}
          </span>
          <span className="rounded-full bg-(--agent-chat-subtle-surface) px-1.5 py-0.5 text-[10px] text-muted-foreground">
            {source.attached
              ? t("agentChat.composer.sourceContext.libraryAttached")
              : t("agentChat.composer.sourceContext.libraryAttach")}
          </span>
        </span>
      </span>
    </button>
  )
}
