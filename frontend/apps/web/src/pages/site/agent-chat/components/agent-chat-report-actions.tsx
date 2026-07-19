import { Download, FileText, FolderOpen } from "lucide-react"
import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"

export interface AgentChatReportActionsProps {
  canUseConversation: boolean
  isExportingTranscript: boolean
  isGeneratingReport: boolean
  isStreaming: boolean
  leading?: ReactNode
  onExportReport: () => void
  onExportTranscript: () => void
  onOpenReportCenter: () => void
}

export function AgentChatReportActions({
  canUseConversation,
  isExportingTranscript,
  isGeneratingReport,
  isStreaming,
  leading,
  onExportReport,
  onExportTranscript,
  onOpenReportCenter,
}: AgentChatReportActionsProps) {
  const { t } = useTranslation()
  const disabled = !canUseConversation || isStreaming
  const isBusy = isExportingTranscript || isGeneratingReport

  return (
    <div
      className="relative z-20 shrink-0 border-b border-(--document-card-border) bg-(image:--document-card-background) px-4 py-2 shadow-sm backdrop-blur-md sm:px-7"
      data-agent-chat-status-card="true"
    >
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div className="min-w-0 flex-1">
          {leading ?? (
            <span className="text-xs font-medium text-muted-foreground">
              {t("agentChat.reportActions.currentConversation")}
            </span>
          )}
        </div>
        <div className="flex min-w-0 flex-wrap items-center justify-end gap-1.5">
          <Button
            type="button"
            size="xs"
            variant="ghost"
            disabled={disabled || isExportingTranscript}
            onClick={onExportTranscript}
          >
            <Download className="size-3.5" />
            <span>
              {isExportingTranscript
                ? t("agentChat.reportActions.exportingTranscript")
                : t("agentChat.reportActions.exportTranscript")}
            </span>
          </Button>
          <Button
            type="button"
            size="xs"
            variant="ghost"
            disabled={disabled || isGeneratingReport}
            onClick={onExportReport}
          >
            <FileText className="size-3.5" />
            <span>
              {isGeneratingReport
                ? t("agentChat.reportActions.generatingReport")
                : t("agentChat.reportActions.exportReport")}
            </span>
          </Button>
          <span className="h-4 w-px bg-border" />
          <Button
            type="button"
            size="xs"
            variant="ghost"
            disabled={disabled || isBusy}
            onClick={onOpenReportCenter}
          >
            <FolderOpen className="size-3.5" />
            <span>{t("agentChat.reportActions.openReportCenter")}</span>
          </Button>
        </div>
      </div>
    </div>
  )
}
