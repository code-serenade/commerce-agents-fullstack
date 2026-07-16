import { useState } from "react"
import { useTranslation } from "react-i18next"
import { getOutputArtifactDisplay } from "../view-model/output-artifact-display-view-model"
import type { AgentOutputBackendState } from "../use-agent-output-page"
import { OutputReportDetail } from "./output-report-detail"
import { OutputReportList } from "./output-report-list"

export function OutputBackendDeliverables({
  backend,
}: {
  backend: AgentOutputBackendState
}) {
  const { t } = useTranslation()
  const [mobileView, setMobileView] = useState<"list" | "detail">("list")
  const selectedArtifact = backend.selectedArtifact
  const artifactExport = backend.artifactExport
  const content =
    artifactExport?.content ||
    selectedArtifact?.content ||
    selectedArtifact?.preview ||
    ""
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

  const openArtifact = (artifactId: string) => {
    backend.setSelectedArtifactId(artifactId)
    setMobileView("detail")
  }

  return (
    <section
      className={`flex size-full min-h-0 flex-1 flex-col overflow-hidden bg-muted/25 dark:bg-background ${
        mobileView === "detail" ? "p-0 md:p-4" : "p-3 sm:p-4"
      }`}
    >
      <div
        className={`grid min-h-0 flex-1 grid-cols-1 grid-rows-1 overflow-hidden border border-(--app-border) bg-white md:grid-cols-[19rem_minmax(0,1fr)] xl:grid-cols-[20rem_minmax(0,1fr)] dark:bg-background ${
          mobileView === "detail"
            ? "rounded-none border-x-0 border-b-0 md:rounded-lg md:border"
            : "rounded-lg"
        }`}
      >
        <OutputReportList
          backend={backend}
          mobileHidden={mobileView === "detail"}
          onRefresh={backend.refetch}
          onSelectArtifact={openArtifact}
        />
        <OutputReportDetail
          artifactExport={artifactExport}
          content={content}
          isMarkdownContent={isMarkdownArtifact(
            artifactExport?.format || selectedArtifact?.format,
            artifactExport?.mimeType || selectedArtifact?.mimeType
          )}
          selectedArtifact={selectedArtifact}
          selectedDisplay={selectedDisplay}
          mobileHidden={mobileView === "list"}
          onMobileBack={() => setMobileView("list")}
          onCopy={copyExport}
          onDownload={downloadExport}
        />
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
