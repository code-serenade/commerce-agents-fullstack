import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { useMutation } from "@tanstack/react-query"
import {
  exportConversationTranscriptApi,
  generateConversationReportApi,
  type GenerateConversationReportResponse,
} from "@/api"
import {
  downloadTextFile,
  notifyError,
  notifySuccess,
} from "@/pages/site/commerce/components/commerce-utils"
import {
  createConversationReportRequest,
  defaultConversationReportDraft,
} from "../../agent-output/view-model/conversation-report-view-model"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import {
  createAgentOutputReportUrl,
  getAgentChatReportRouteContext,
} from "./agent-chat-report-actions"

export interface AgentChatReportActionsState {
  canUseConversation: boolean
  exportReport: () => void
  exportTranscript: () => void
  isGeneratingReport: boolean
  isExportingTranscript: boolean
  openReportCenter: () => void
}

export function useAgentChatReportActions(input: {
  conversationId: string | null
  isStreaming: boolean
  runtimeEvents: AgentRuntimeEvent[]
}): AgentChatReportActionsState {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const reportRouteContext = getAgentChatReportRouteContext({
    conversationId: input.conversationId,
    runtimeEvents: input.runtimeEvents,
  })
  const reportRequestContext = reportRouteContext
    ? {
        conversationId: reportRouteContext.conversationId,
        runId: reportRouteContext.runId ?? null,
        workspaceId: reportRouteContext.workspaceId ?? null,
      }
    : null
  const transcriptExportMutation = useMutation({
    mutationFn: () =>
      exportConversationTranscriptApi(input.conversationId!, {
        format: "markdown",
        includeFiles: true,
        includeRuntimeEvents: true,
        scope: "conversation_all",
      }),
    onSuccess: (response) => {
      downloadTextFile({
        content: response.content,
        fileName: response.fileName,
        mimeType: response.mimeType,
      })
      notifySuccess(t("agentChat.reportActions.transcriptExported"))
    },
    onError: () => {
      notifyError(t("agentChat.reportActions.transcriptFailed"))
    },
  })
  const reportExportMutation = useMutation({
    mutationFn: () =>
      generateConversationReportApi(
        reportRouteContext!.conversationId,
        createConversationReportRequest(
          defaultConversationReportDraft,
          reportRequestContext!
        )
      ),
    onSuccess: (response) => {
      downloadGeneratedReport(response)
      notifySuccess(t("agentChat.reportActions.reportExported"))
    },
    onError: () => {
      notifyError(t("agentChat.reportActions.reportFailed"))
    },
  })

  return {
    canUseConversation: Boolean(reportRouteContext),
    exportReport: () => {
      if (
        !reportRouteContext ||
        input.isStreaming ||
        reportExportMutation.isPending
      ) {
        return
      }
      reportExportMutation.mutate()
    },
    exportTranscript: () => {
      if (
        !input.conversationId ||
        input.isStreaming ||
        transcriptExportMutation.isPending
      ) {
        return
      }
      transcriptExportMutation.mutate()
    },
    isGeneratingReport: reportExportMutation.isPending,
    isExportingTranscript: transcriptExportMutation.isPending,
    openReportCenter: () => {
      if (!reportRouteContext || input.isStreaming) return
      navigate(createAgentOutputReportUrl(reportRouteContext))
    },
  }
}

function downloadGeneratedReport(response: GenerateConversationReportResponse) {
  downloadTextFile({
    content: response.export.content,
    fileName: response.export.fileName,
    mimeType: response.export.mimeType,
  })
}
