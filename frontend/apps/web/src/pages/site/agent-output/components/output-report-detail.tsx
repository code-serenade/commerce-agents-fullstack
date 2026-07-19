import { ArrowLeft, Copy, Download } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type { WorkspaceArtifactExportResponse } from "@/api"
import { MutedText } from "@/pages/site/commerce/components/commerce-ui"
import type { AgentOutputDeliverable } from "../use-agent-output-page"
import {
  formatArtifactFormat,
  formatContentLength,
  formatOutputBackendNotice,
  type OutputArtifactDisplay,
} from "../view-model/output-artifact-display-view-model"
import { EmptyState } from "./output-backend-deliverable-blocks"
import { OutputMarkdownContent } from "./output-markdown-content"
import { OutputValidationBadge } from "./output-validation-badge"

export interface OutputReportDetailProps {
  artifactExport: WorkspaceArtifactExportResponse | null
  content: string
  isMarkdownContent: boolean
  selectedArtifact: AgentOutputDeliverable | null
  selectedDisplay: OutputArtifactDisplay | null
  mobileHidden: boolean
  onMobileBack: () => void
  onCopy: () => void
  onDownload: () => void
}

export function OutputReportDetail({
  artifactExport,
  content,
  isMarkdownContent,
  selectedArtifact,
  selectedDisplay,
  mobileHidden,
  onMobileBack,
  onCopy,
  onDownload,
}: OutputReportDetailProps) {
  const { t } = useTranslation()

  if (!selectedArtifact || !selectedDisplay) {
    return (
      <section
        className={`min-h-0 min-w-0 bg-white p-4 md:block dark:bg-background ${
          mobileHidden ? "hidden" : "block"
        }`}
      >
        <EmptyState
          title={t("agentOutput.backend.noSelectionTitle")}
          detail={t("agentOutput.backend.noSelectionDescription")}
        />
      </section>
    )
  }

  return (
    <section
      className={`min-h-0 min-w-0 flex-col overflow-hidden bg-white md:flex dark:bg-background ${
        mobileHidden ? "hidden" : "flex"
      }`}
    >
      <div className="flex shrink-0 items-center border-b border-(--app-border) bg-white px-4 py-2.5 md:hidden dark:bg-background">
        <Button
          type="button"
          size="sm"
          variant="link"
          className="text-primary hover:text-primary active:text-primary"
          onClick={onMobileBack}
        >
          <ArrowLeft className="size-4" />
          {t("agentOutput.backend.backToList")}
        </Button>
      </div>

      <header className="flex min-w-0 shrink-0 flex-col gap-2.5 border-b border-(--app-border) bg-muted/15 px-4 py-3 md:gap-3 md:bg-white md:px-5 lg:flex-row lg:items-start lg:justify-between dark:bg-background">
        <div className="min-w-0">
          <h2 className="line-clamp-2 text-base/6 font-semibold text-foreground md:block md:truncate">
            {selectedDisplay.title}
          </h2>
          <div className="mt-1.5 flex flex-wrap items-center gap-2 md:mt-1">
            <OutputValidationBadge
              label={selectedDisplay.validationStatus}
              status={selectedArtifact.validationStatus}
            />
            <span className="hidden md:inline-flex">
              <MutedText>
                {selectedDisplay.type} / {selectedDisplay.source}
              </MutedText>
            </span>
          </div>
        </div>
        <div className="grid shrink-0 grid-cols-2 gap-2 md:flex md:flex-wrap">
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="md:border-transparent md:bg-transparent md:text-foreground md:hover:bg-muted"
            disabled={!content}
            onClick={onCopy}
          >
            <Copy className="size-4" />
            {t("agentOutput.backend.copy")}
          </Button>
          <Button
            type="button"
            size="sm"
            variant="primary"
            disabled={!content}
            onClick={onDownload}
          >
            <Download className="size-4" />
            {t("agentOutput.backend.download")}
          </Button>
        </div>
      </header>

      <div className="hidden shrink-0 flex-wrap items-center gap-x-4 gap-y-1 border-b border-(--app-border) bg-white px-5 py-2 text-xs text-muted-foreground md:flex dark:bg-background">
        <ReportMeta
          label={t("agentOutput.backend.meta.workspace")}
          value={selectedArtifact.workspace.title}
        />
        <ReportMeta
          label={t("agentOutput.backend.meta.source")}
          value={selectedDisplay.source}
        />
        <ReportMeta
          label={t("agentOutput.backend.meta.format")}
          value={formatArtifactFormat(
            t,
            artifactExport?.format || selectedArtifact.format
          )}
        />
        <ReportMeta
          label={t("agentOutput.backend.meta.validation")}
          value={selectedDisplay.validationStatus}
        />
        <ReportMeta
          label={t("agentOutput.backend.meta.size")}
          value={formatContentLength(
            t,
            artifactExport?.contentChars ?? content.length
          )}
        />
      </div>

      {artifactExport?.warnings?.length ? (
        <div className="shrink-0 border-b border-amber-300 bg-amber-50 px-5 py-2 text-sm text-amber-900 dark:border-amber-700 dark:bg-amber-950/30 dark:text-amber-100">
          {artifactExport.warnings
            .map((warning) => formatOutputBackendNotice(t, warning))
            .join(" / ")}
        </div>
      ) : null}

      <div className="flex min-h-0 flex-1 flex-col overflow-hidden bg-white dark:bg-background">
        <div className="shrink-0 border-b border-(--app-border) bg-white px-4 py-2 md:hidden dark:bg-background">
          <p className="text-xs font-semibold text-muted-foreground">
            {selectedDisplay.format}
          </p>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto md:[scrollbar-gutter:stable]">
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
    </section>
  )
}

function ReportMeta({ label, value }: { label: string; value: string }) {
  return (
    <span className="inline-flex min-w-0 items-center gap-1.5">
      <span>{label}</span>
      <span className="max-w-56 truncate font-medium text-foreground">
        {value}
      </span>
    </span>
  )
}
