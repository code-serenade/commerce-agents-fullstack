import { Copy, Download, FileText, RefreshCcw } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button, NativeSelect } from "@workspace/ui-components"
import {
  MutedText,
  StatusBadge,
  toneForStatus,
} from "@/pages/site/commerce/components/commerce-ui"
import {
  formatContentLength,
  formatArtifactFormat,
  getOutputArtifactDisplay,
  formatOutputBackendNotice,
} from "../view-model/output-artifact-display-view-model"
import type { AgentOutputBackendState } from "../use-agent-output-page"
import {
  ArtifactMeta,
  EmptyState,
  LoadingState,
} from "./output-backend-deliverable-blocks"
import { OutputMarkdownContent } from "./output-markdown-content"

export function OutputBackendDeliverables({
  backend,
}: {
  backend: AgentOutputBackendState
}) {
  const { t } = useTranslation()
  const selectedArtifact = backend.selectedArtifact
  const artifactExport = backend.artifactExport
  const content =
    artifactExport?.content ||
    selectedArtifact?.content ||
    selectedArtifact?.preview ||
    ""
  const selectedId = selectedArtifact?.id ?? backend.selectedArtifactId ?? ""
  const selectedDisplay = selectedArtifact
    ? getOutputArtifactDisplay(t, selectedArtifact)
    : null
  const isMarkdownContent = isMarkdownArtifact(
    artifactExport?.format || selectedArtifact?.format,
    artifactExport?.mimeType || selectedArtifact?.mimeType
  )

  const copyExport = () => {
    if (!content) return
    void navigator.clipboard?.writeText(content)
  }

  const downloadExport = () => {
    if (!content) return
    const blob = new Blob([content], {
      type:
        artifactExport?.mimeType ||
        selectedArtifact?.mimeType ||
        "text/plain;charset=utf-8",
    })
    const url = URL.createObjectURL(blob)
    const anchor = document.createElement("a")
    anchor.href = url
    anchor.download =
      artifactExport?.fileName ||
      selectedArtifact?.fileName ||
      `${selectedDisplay?.downloadBaseName ?? t("agentOutput.backend.downloadFileName")}.txt`
    anchor.click()
    URL.revokeObjectURL(url)
  }

  return (
    <section className="overflow-hidden rounded-lg border border-(--document-intro-card-border) bg-(image:--document-intro-card-background) shadow-(--document-intro-card-shadow) backdrop-blur-md">
      <div className="flex min-w-0 flex-col gap-3 border-b border-(--document-intro-card-border) p-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-1">
          <p className="text-xs font-semibold tracking-normal text-(--app-active-text)">
            {t("agentOutput.backend.eyebrow")}
          </p>
          <h1 className="text-2xl/tight font-semibold text-foreground">
            {t("agentOutput.backend.title")}
          </h1>
          <p className="max-w-3xl text-sm/6 text-muted-foreground">
            {t("agentOutput.backend.description")}
          </p>
        </div>
        <div className="flex shrink-0 flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={backend.refetch}
          >
            <RefreshCcw className="size-4" />
            {t("agentOutput.backend.refresh")}
          </Button>
        </div>
      </div>

      <div className="grid gap-4 bg-muted/20 p-4 xl:grid-cols-[minmax(21rem,0.74fr)_minmax(0,1.26fr)]">
        <div className="min-w-0 overflow-hidden rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) shadow-(--document-card-shadow) backdrop-blur-md">
          <div className="mb-3 flex min-w-0 items-center justify-between gap-2">
            <div className="min-w-0 px-3 pt-3">
              <h2 className="truncate text-base font-semibold">
                {t("agentOutput.backend.listTitle")}
              </h2>
              <MutedText>
                {t("agentOutput.backend.listDesc", {
                  artifacts: backend.deliverables.length,
                  workspaces: backend.workspaces.length,
                })}
              </MutedText>
            </div>
          </div>

          {backend.loading ? (
            <div className="px-3 pb-3">
              <LoadingState label={t("agentOutput.backend.loading")} />
            </div>
          ) : backend.errorMessage ? (
            <div className="px-3 pb-3">
              <EmptyState
                title={t("agentOutput.backend.errorTitle")}
                detail={formatOutputBackendNotice(t, backend.errorMessage)}
              />
            </div>
          ) : backend.deliverables.length ? (
            <div className="space-y-3 px-3 pb-3">
              <div className="rounded-lg border border-(--document-card-border) bg-muted/35 p-1.5">
                <NativeSelect
                  value={selectedId}
                  options={backend.deliverables.map((artifact) => ({
                    label: getOutputArtifactDisplay(t, artifact).title,
                    value: artifact.id,
                  }))}
                  onValueChange={backend.setSelectedArtifactId}
                />
              </div>
              <div className="h-[min(41rem,calc(100svh-18rem))] min-h-80 space-y-2 overflow-y-auto pr-1 [scrollbar-gutter:stable]">
                {backend.deliverables.map((artifact) => {
                  const active = artifact.id === selectedId
                  const display = getOutputArtifactDisplay(t, artifact)

                  return (
                    <button
                      key={artifact.id}
                      type="button"
                      className={`group block w-full min-w-0 rounded-lg border p-3 text-left shadow-sm transition ${
                        active
                          ? "border-primary/45 bg-[color-mix(in_oklch,var(--app-active-bg)_34%,var(--app-surface))] text-foreground shadow-[inset_3px_0_0_var(--app-active-text)]"
                          : "border-(--document-card-border) bg-muted/30 hover:border-primary/25 hover:bg-(--shell-sidebar-item-hover-background)"
                      }`}
                      onClick={() => backend.setSelectedArtifactId(artifact.id)}
                    >
                      <span className="flex min-w-0 items-start gap-2">
                        <span
                          className={`mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md border ${
                            active
                              ? "border-primary/35 bg-(--app-active-bg) text-(--app-active-text)"
                              : "border-(--document-card-border) bg-muted/30 text-muted-foreground group-hover:text-foreground"
                          }`}
                        >
                          <FileText className="size-4" />
                        </span>
                        <span className="min-w-0 flex-1">
                          <span className="block truncate text-sm font-semibold">
                            {display.title}
                          </span>
                          <span className="mt-1 block truncate text-xs text-muted-foreground">
                            {artifact.workspace.title}
                          </span>
                        </span>
                      </span>
                      <span className="mt-2 flex flex-wrap items-center gap-2">
                        <StatusBadge
                          tone={toneForStatus(artifact.validationStatus)}
                        >
                          {display.validationStatus}
                        </StatusBadge>
                        <span className="truncate text-xs text-muted-foreground">
                          {display.type} / {display.format}
                        </span>
                      </span>
                    </button>
                  )
                })}
              </div>
            </div>
          ) : (
            <div className="px-3 pb-3">
              <EmptyState
                title={t("agentOutput.backend.emptyTitle")}
                detail={t("agentOutput.backend.emptyDescription")}
              />
            </div>
          )}
        </div>

        <div className="min-w-0 overflow-hidden rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) shadow-(--document-card-shadow) backdrop-blur-md">
          {selectedArtifact && selectedDisplay ? (
            <article className="min-w-0">
              <div className="flex min-w-0 flex-col gap-3 border-b border-(--document-card-border) bg-muted/30 p-4 lg:flex-row lg:items-start lg:justify-between">
                <div className="min-w-0">
                  <h2 className="truncate text-xl font-semibold text-foreground">
                    {selectedDisplay.title}
                  </h2>
                  <div className="mt-1 flex flex-wrap items-center gap-2">
                    <StatusBadge tone={toneForStatus(selectedArtifact.status)}>
                      {selectedDisplay.status}
                    </StatusBadge>
                    <StatusBadge
                      tone={toneForStatus(selectedArtifact.validationStatus)}
                    >
                      {selectedDisplay.validationStatus}
                    </StatusBadge>
                    <MutedText>
                      {selectedDisplay.type} / {selectedDisplay.source}
                    </MutedText>
                  </div>
                </div>
                <div className="flex shrink-0 flex-wrap gap-2">
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={!content}
                    onClick={copyExport}
                  >
                    <Copy className="size-4" />
                    {t("agentOutput.backend.copy")}
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={!content}
                    onClick={downloadExport}
                  >
                    <Download className="size-4" />
                    {t("agentOutput.backend.download")}
                  </Button>
                </div>
              </div>

              <div className="grid gap-2 border-b border-(--document-card-border) bg-muted/25 p-4 text-xs text-muted-foreground sm:grid-cols-2 xl:grid-cols-5">
                <ArtifactMeta
                  label={t("agentOutput.backend.meta.workspace")}
                  value={selectedArtifact.workspace.title}
                />
                <ArtifactMeta
                  label={t("agentOutput.backend.meta.source")}
                  value={selectedDisplay.source}
                />
                <ArtifactMeta
                  label={t("agentOutput.backend.meta.format")}
                  value={formatArtifactFormat(
                    t,
                    artifactExport?.format || selectedArtifact.format
                  )}
                />
                <ArtifactMeta
                  label={t("agentOutput.backend.meta.validation")}
                  value={selectedDisplay.validationStatus}
                />
                <ArtifactMeta
                  label={t("agentOutput.backend.meta.size")}
                  value={formatContentLength(
                    t,
                    artifactExport?.contentChars ?? content.length
                  )}
                />
              </div>

              {artifactExport?.warnings?.length ? (
                <div className="px-4 pt-4">
                  <div className="rounded-lg border border-amber-300 bg-amber-50 px-3 py-2 text-sm text-amber-900 dark:border-amber-700 dark:bg-amber-950/30 dark:text-amber-100">
                    {artifactExport.warnings
                      .map((warning) => formatOutputBackendNotice(t, warning))
                      .join(" / ")}
                  </div>
                </div>
              ) : null}

              <div className="bg-muted/15 p-4">
                <div className="max-h-150 min-h-150 overflow-auto rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) shadow-(--document-card-shadow) [scrollbar-gutter:stable]">
                  <div className="border-b border-(--document-card-border) px-5 py-3">
                    <p className="text-xs font-semibold text-muted-foreground">
                      {selectedDisplay.format}
                    </p>
                  </div>
                  {content ? (
                    isMarkdownContent ? (
                      <OutputMarkdownContent content={content} />
                    ) : (
                      <div className="p-5 text-sm/7 wrap-break-word whitespace-pre-wrap text-foreground">
                        {content}
                      </div>
                    )
                  ) : (
                    <div className="p-5 text-sm/7 wrap-break-word text-muted-foreground">
                      {t("agentOutput.backend.noContent")}
                    </div>
                  )}
                </div>
              </div>
            </article>
          ) : (
            <div className="p-3">
              <EmptyState
                title={t("agentOutput.backend.noSelectionTitle")}
                detail={t("agentOutput.backend.noSelectionDescription")}
              />
            </div>
          )}
        </div>
      </div>
    </section>
  )
}

function isMarkdownArtifact(format?: string | null, mimeType?: string | null) {
  const normalizedFormat = String(format ?? "")
    .trim()
    .toLowerCase()
  const normalizedMimeType = String(mimeType ?? "")
    .trim()
    .toLowerCase()

  return (
    normalizedFormat === "markdown" ||
    normalizedFormat === "md" ||
    normalizedMimeType === "text/markdown" ||
    normalizedMimeType === "application/markdown"
  )
}
