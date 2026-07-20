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
      className="relative z-20 shrink-0 px-4 pt-3 sm:px-7 sm:pt-4"
      data-agent-chat-report-actions="true"
    >
      <div className="mx-auto flex w-full max-w-4xl min-w-0 flex-wrap items-center justify-between gap-x-3 gap-y-2 rounded-2xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-3 py-2 shadow-(--agent-chat-floating-shadow) backdrop-blur-xl">
        <div className="min-w-0 flex-1 basis-[20rem] text-left">
          {leading ?? (
            <span className="text-xs font-medium text-muted-foreground">
              {t("agentChat.reportActions.currentConversation")}
            </span>
          )}
        </div>
        <div className="flex min-w-0 flex-1 basis-[18rem] flex-wrap items-center justify-start gap-1 sm:justify-end">
          <Button
            type="button"
            size="xs"
            variant="ghost"
            disabled={disabled || isExportingTranscript}
            onClick={onExportTranscript}
          >
            <Download className="size-3.5" />
            <span className="whitespace-nowrap">
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
            <span className="whitespace-nowrap">
              {isGeneratingReport
                ? t("agentChat.reportActions.generatingReport")
                : t("agentChat.reportActions.exportReport")}
            </span>
          </Button>
          <span className="h-4 w-px shrink-0 bg-border" />
          <Button
            type="button"
            size="xs"
            variant="ghost"
            disabled={disabled || isBusy}
            onClick={onOpenReportCenter}
          >
            <FolderOpen className="size-3.5" />
            <span className="whitespace-nowrap">
              {t("agentChat.reportActions.openReportCenter")}
            </span>
          </Button>
        </div>
      </div>
    </div>
  )
}
