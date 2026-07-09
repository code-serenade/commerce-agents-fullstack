import { AlertTriangle, DatabaseZap, FileText, Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentContextPreviewResponse } from "../agent-chat-data"
import {
  getAgentContextPreviewModel,
  type AgentContextPreviewRefStatus,
} from "../view-model/agent-context-preview-view-model"
import { AgentChatContextPreviewAuditSummary } from "./agent-chat-context-preview-audit-summary"
import {
  ComposerPanelFrame,
  composerPanelSecondaryButtonClassName,
} from "./composer-panel-layout"

export function AgentChatContextPreviewPanel({
  canPreview,
  error,
  isPreviewing,
  onPreview,
  preview,
}: {
  canPreview: boolean
  error: string | null
  isPreviewing: boolean
  onPreview: () => void
  preview: AgentContextPreviewResponse | null
}) {
  const { t } = useTranslation()
  const model = getAgentContextPreviewModel(preview)
  const buttonLabel = model.hasPreview
    ? t("agentChat.composer.contextPreview.refresh")
    : t("agentChat.composer.contextPreview.run")

  return (
    <section className="grid gap-3">
      <ComposerPanelFrame
        title={t("agentChat.composer.contextPreview.panelTitle")}
        description={t("agentChat.composer.contextPreview.panelHint")}
        actions={
          <button
            type="button"
            disabled={!canPreview || isPreviewing}
            onClick={onPreview}
            className={composerPanelSecondaryButtonClassName}
          >
            {isPreviewing ? (
              <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
            ) : (
              <DatabaseZap className="size-3.5" aria-hidden="true" />
            )}
            <span>{buttonLabel}</span>
          </button>
        }
      >
        {isPreviewing ? (
          <div className="rounded-md border border-border bg-muted/40 px-3 py-2 text-xs text-muted-foreground">
            {t("agentChat.composer.contextPreview.loading")}
          </div>
        ) : null}

        {error ? (
          <div className="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            <AlertTriangle
              className="mt-0.5 size-3.5 shrink-0"
              aria-hidden="true"
            />
            <span>{error}</span>
          </div>
        ) : null}

        {!model.hasPreview && !isPreviewing ? (
          <div className="rounded-md border border-dashed border-border bg-muted/30 p-3 text-xs/5 text-muted-foreground">
            {t("agentChat.composer.contextPreview.empty")}
          </div>
        ) : null}

        {model.hasPreview ? (
          <div className="grid gap-3">
            <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
              <ContextPreviewCount
                label={t("agentChat.composer.contextPreview.counts.included")}
                value={model.readCounts.included}
              />
              <ContextPreviewCount
                label={t(
                  "agentChat.composer.contextPreview.counts.metadataOnly"
                )}
                value={model.readCounts.metadataOnly}
              />
              <ContextPreviewCount
                label={t("agentChat.composer.contextPreview.counts.omitted")}
                value={model.readCounts.omitted}
              />
              <ContextPreviewCount
                label={t("agentChat.composer.contextPreview.counts.memory")}
                value={model.readCounts.memory}
              />
            </div>

            <AgentChatContextPreviewAuditSummary summary={model.auditSummary} />

            <section className="grid gap-2">
              <h3 className="text-xs font-semibold text-foreground">
                {t("agentChat.composer.contextPreview.refsTitle")}
              </h3>
              {model.refRows.length ? (
                <div className="grid gap-1.5">
                  {model.refRows.map((row) => (
                    <div
                      key={row.id}
                      className="flex min-w-0 items-center gap-2 rounded-md border border-border bg-card px-2.5 py-1.5"
                    >
                      <FileText className="size-3.5 shrink-0 text-muted-foreground" />
                      <span className="truncate text-xs text-foreground">
                        {row.ref}
                      </span>
                      <span className={refStatusClassName(row.status)}>
                        {t(
                          `agentChat.composer.contextPreview.refStatus.${row.status}`
                        )}
                      </span>
                    </div>
                  ))}
                  {model.hiddenRefCount > 0 ? (
                    <p className="text-[11px] text-muted-foreground">
                      {t("agentChat.composer.contextPreview.moreRefs", {
                        count: model.hiddenRefCount,
                      })}
                    </p>
                  ) : null}
                </div>
              ) : (
                <p className="rounded-md border border-border bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
                  {t("agentChat.composer.contextPreview.refsEmpty")}
                </p>
              )}
            </section>

            <section className="grid gap-2">
              <h3 className="text-xs font-semibold text-foreground">
                {t("agentChat.composer.contextPreview.memoryTitle")}
              </h3>
              {model.memoryRows.length ? (
                <div className="grid gap-1.5">
                  {model.memoryRows.map((row) => (
                    <div
                      key={row.id}
                      className="rounded-md border border-border bg-card px-2.5 py-1.5"
                    >
                      <div className="flex min-w-0 items-center justify-between gap-2">
                        <span className="truncate text-xs font-medium text-foreground">
                          {row.name}
                        </span>
                        <span className="shrink-0 text-[11px] text-muted-foreground">
                          {t("agentChat.composer.contextPreview.charCount", {
                            count: row.charCount,
                          })}
                        </span>
                      </div>
                      <p className="mt-1 truncate text-[11px] text-muted-foreground">
                        {formatMemoryMeta(row.kind, row.tags)}
                        {row.truncated
                          ? t("agentChat.composer.contextPreview.truncated")
                          : ""}
                      </p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="rounded-md border border-border bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
                  {t("agentChat.composer.contextPreview.memoryEmpty")}
                </p>
              )}
            </section>

            {model.warnings.length ? (
              <section className="grid gap-2">
                <h3 className="text-xs font-semibold text-foreground">
                  {t("agentChat.composer.contextPreview.warningTitle")}
                </h3>
                <div className="grid gap-1.5">
                  {model.warnings.map((warning) => (
                    <p
                      key={warning}
                      className="rounded-md border border-amber-300/50 bg-amber-50 px-3 py-2 text-xs text-amber-800 dark:border-amber-400/30 dark:bg-amber-500/10 dark:text-amber-100"
                    >
                      {warning}
                    </p>
                  ))}
                </div>
              </section>
            ) : null}
          </div>
        ) : null}
      </ComposerPanelFrame>
    </section>
  )
}

function ContextPreviewCount({
  label,
  value,
}: {
  label: string
  value: number
}) {
  return (
    <div className="rounded-md border border-border bg-card px-3 py-2">
      <div className="text-base/5 font-semibold text-foreground">{value}</div>
      <div className="mt-1 truncate text-[11px] text-muted-foreground">
        {label}
      </div>
    </div>
  )
}

function refStatusClassName(status: AgentContextPreviewRefStatus) {
  const base = "shrink-0 rounded-full px-2 py-0.5 text-[10px] font-medium"
  if (status === "included") {
    return `${base} bg-emerald-50 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-200`
  }
  if (status === "metadataOnly") {
    return `${base} bg-sky-50 text-sky-700 dark:bg-sky-500/15 dark:text-sky-200`
  }
  return `${base} bg-muted text-muted-foreground`
}

function formatMemoryMeta(kind: string, tags: string[]) {
  return [kind, ...tags].filter(Boolean).join(" / ")
}
