import { Copy, Download, RefreshCcw } from "lucide-react"
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
import { ArtifactMeta, EmptyState, LoadingState } from "./output-backend-deliverable-blocks"

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

  const copyExport = () => {
    if (!content) return
    void navigator.clipboard?.writeText(content)
  }

  const downloadExport = () => {
    if (!content) return
    const blob = new Blob([content], {
      type: artifactExport?.mimeType || selectedArtifact?.mimeType || "text/plain;charset=utf-8",
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
    <section className="rounded-lg border border-(--app-border) bg-(--app-surface)">
      <div className="flex min-w-0 flex-col gap-3 border-b border-(--app-border) p-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0">
          <p className="text-xs font-medium tracking-normal text-muted-foreground">
            {t("agentOutput.backend.eyebrow")}
          </p>
          <h1 className="mt-1 text-2xl/tight font-semibold text-foreground">
            {t("agentOutput.backend.title")}
          </h1>
          <p className="mt-1 max-w-3xl text-sm/6 text-muted-foreground">
            {t("agentOutput.backend.description")}
          </p>
        </div>
        <div className="flex shrink-0 flex-wrap items-center gap-2">
          <Button type="button" variant="outline" size="sm" onClick={backend.refetch}>
            <RefreshCcw className="size-4" />
            {t("agentOutput.backend.refresh")}
          </Button>
        </div>
      </div>

      <div className="grid gap-4 p-4 xl:grid-cols-[minmax(20rem,0.82fr)_minmax(0,1.18fr)]">
        <div className="min-w-0 rounded-lg border border-(--app-border) bg-background p-3">
          <div className="mb-3 flex min-w-0 items-center justify-between gap-2">
            <div className="min-w-0">
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
            <LoadingState label={t("agentOutput.backend.loading")} />
          ) : backend.errorMessage ? (
            <EmptyState
              title={t("agentOutput.backend.errorTitle")}
              detail={formatOutputBackendNotice(t, backend.errorMessage)}
            />
          ) : backend.deliverables.length ? (
            <div className="space-y-3">
              <NativeSelect
                value={selectedId}
                options={backend.deliverables.map((artifact) => ({
                  label: getOutputArtifactDisplay(t, artifact).title,
                  value: artifact.id,
                }))}
                onValueChange={backend.setSelectedArtifactId}
              />
              <div className="max-h-128 space-y-2 overflow-y-auto pr-1">
                {backend.deliverables.map((artifact) => {
                  const active = artifact.id === selectedId
                  const display = getOutputArtifactDisplay(t, artifact)

                  return (
                    <button
                      key={artifact.id}
                      type="button"
                      className={`block w-full min-w-0 rounded-lg border px-3 py-2 text-left transition-colors ${
                        active
                          ? "border-blue-300 bg-blue-50 text-blue-950 dark:border-blue-700 dark:bg-blue-950/35 dark:text-blue-50"
                          : "border-(--app-border) bg-card hover:bg-muted"
                      }`}
                      onClick={() => backend.setSelectedArtifactId(artifact.id)}
                    >
                      <span className="block truncate text-sm font-medium">
                        {display.title}
                      </span>
                      <span className="mt-1 block truncate text-xs text-muted-foreground">
                        {artifact.workspace.title}
                      </span>
                      <span className="mt-2 flex flex-wrap items-center gap-2">
                        <StatusBadge tone={toneForStatus(artifact.validationStatus)}>
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
            <EmptyState
              title={t("agentOutput.backend.emptyTitle")}
              detail={t("agentOutput.backend.emptyDescription")}
            />
          )}
        </div>

        <div className="min-w-0 rounded-lg border border-(--app-border) bg-background p-3">
          {selectedArtifact && selectedDisplay ? (
            <div className="space-y-4">
              <div className="flex min-w-0 flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                <div className="min-w-0">
                  <h2 className="truncate text-lg font-semibold">
                    {selectedDisplay.title}
                  </h2>
                  <div className="mt-1 flex flex-wrap items-center gap-2">
                    <StatusBadge tone={toneForStatus(selectedArtifact.status)}>
                      {selectedDisplay.status}
                    </StatusBadge>
                    <StatusBadge tone={toneForStatus(selectedArtifact.validationStatus)}>
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

              <div className="grid gap-2 text-xs text-muted-foreground sm:grid-cols-2 xl:grid-cols-3">
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
                <div className="rounded-lg border border-amber-300 bg-amber-50 px-3 py-2 text-sm text-amber-900 dark:border-amber-700 dark:bg-amber-950/30 dark:text-amber-100">
                  {artifactExport.warnings
                    .map((warning) => formatOutputBackendNotice(t, warning))
                    .join(" / ")}
                </div>
              ) : null}

              <div className="max-h-136 overflow-auto rounded-lg border border-(--app-border) bg-muted/30 p-3">
                <pre className="wrap-break-word whitespace-pre-wrap text-sm/6 text-foreground">
                  {content || t("agentOutput.backend.noContent")}
                </pre>
              </div>
            </div>
          ) : (
            <EmptyState
              title={t("agentOutput.backend.noSelectionTitle")}
              detail={t("agentOutput.backend.noSelectionDescription")}
            />
          )}
        </div>
      </div>
    </section>
  )
}
