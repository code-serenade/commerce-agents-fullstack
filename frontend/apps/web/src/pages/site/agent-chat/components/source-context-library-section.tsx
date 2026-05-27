import {
  CheckCircle2,
  FilePlus2,
  FileText,
  FolderOpen,
  Loader2,
} from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { SearchInput } from "@workspace/ui-components"
import type { AgentChatSourceLibraryItem } from "../domain/agent-chat-context-selection"

const LIBRARY_PAGE_SIZE = 6

export function SourceContextLibrarySection({
  disabled,
  isLoading,
  onAttachExistingSource,
  onManageSources,
  sourceLibrary,
  sourceLibraryError,
}: {
  disabled: boolean
  isLoading: boolean
  onAttachExistingSource?: (fileId: string) => void
  onManageSources?: () => void
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
    sourceLibrary.length > 0 ||
    Boolean(onManageSources)

  if (!visible) return null

  return (
    <div className="grid min-w-0 gap-2 border-t border-border pt-3">
      <div className="min-w-0">
        <p className="text-[11px] font-semibold text-foreground">
          {t("agentChat.composer.sourceContext.librarySection")}
        </p>
        <p className="mt-0.5 text-[11px] text-muted-foreground">
          {t("agentChat.composer.sourceContext.libraryHint")}
        </p>
      </div>
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
        <div className="rounded-lg border border-destructive/30 bg-destructive/10 p-2 text-xs text-destructive">
          {sourceLibraryError}
        </div>
      ) : null}
      {isLoading ? (
        <div className="flex items-center gap-2 rounded-lg border border-border bg-background/60 p-3 text-xs text-muted-foreground">
          <Loader2 className="size-4 animate-spin" />
          <span>{t("agentChat.composer.sourceContext.libraryLoading")}</span>
        </div>
      ) : null}
      {!isLoading && sourceLibrary.length && filteredLibrary.length ? (
        <div className="grid min-w-0 gap-2 sm:grid-cols-2">
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
        <div className="rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
          {t("agentChat.composer.sourceContext.libraryNoSearchResult")}
        </div>
      ) : null}
      {!isLoading && hiddenCount > 0 ? (
        <div className="flex justify-center">
          <button
            type="button"
            disabled={disabled}
            onClick={() =>
              setVisibleCount((current) => current + LIBRARY_PAGE_SIZE)
            }
            className="rounded-full border border-border bg-background px-3 py-1 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          >
            {t("agentChat.composer.sourceContext.showMoreLibrary", {
              count: Math.min(hiddenCount, LIBRARY_PAGE_SIZE),
            })}
          </button>
        </div>
      ) : null}
      {!isLoading && !sourceLibraryError && sourceLibrary.length === 0 ? (
        <div className="flex min-w-0 flex-wrap items-center justify-between gap-2 rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
          <div className="flex min-w-0 items-start gap-2">
            <FolderOpen className="mt-0.5 size-4 shrink-0" />
            <span className="min-w-0">
              {t("agentChat.composer.sourceContext.libraryEmpty")}
            </span>
          </div>
          <button
            type="button"
            disabled={disabled || !onManageSources}
            onClick={onManageSources}
            className="inline-flex h-7 shrink-0 items-center gap-1.5 rounded-md border border-border bg-background px-2.5 text-[11px] font-medium text-foreground transition-colors hover:bg-muted disabled:cursor-not-allowed disabled:opacity-50"
          >
            <FolderOpen className="size-3.5" />
            {t("agentChat.composer.sourceContext.libraryEmptyAction")}
          </button>
        </div>
      ) : null}
    </div>
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
          ? "border-emerald-400/50 bg-emerald-50 text-emerald-800 dark:bg-emerald-500/15 dark:text-emerald-100"
          : "border-border bg-background/70 text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={source.title}
    >
      <span
        className={`mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[4px] border ${
          source.attached
            ? "border-emerald-500 bg-emerald-500 text-white"
            : "border-border bg-background text-muted-foreground"
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
          <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
            {t(`agentChat.composer.sourceContext.libraryStatus.${statusKey}`)}
          </span>
          <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
            {source.attached
              ? t("agentChat.composer.sourceContext.libraryAttached")
              : t("agentChat.composer.sourceContext.libraryAttach")}
          </span>
        </span>
      </span>
    </button>
  )
}
