import { useState } from "react"
import { useTranslation } from "react-i18next"
import { useMutation } from "@tanstack/react-query"
import {
  exportConversationTranscriptApi,
  generateConversationReportApi,
  type ConversationTranscriptExportResponse,
  type GenerateConversationReportResponse,
  type WorkspaceArtifactExportResponse,
  type WorkspaceArtifactResponse,
} from "@/api"
import {
  downloadTextFile,
  notifyError,
  notifySuccess,
} from "@/pages/site/commerce/components/commerce-utils"
import {
  createConversationReportRequest,
  type ConversationReportContext,
  type ConversationReportDraft,
} from "./conversation-report-view-model"

export interface ConversationReportActionsInput {
  onGenerated: (response: GenerateConversationReportResponse) => void
  reportContext: ConversationReportContext
}

export interface ConversationReportActionsState {
  artifactExport: WorkspaceArtifactExportResponse | null
  canExportTranscript: boolean
  canGenerateReport: boolean
  closeReportDialog: () => void
  exportTranscript: () => void
  generatedReportExport: GenerateConversationReportResponse["export"] | null
  generateReport: (draft: ConversationReportDraft) => void
  isGeneratingReport: boolean
  isReportDialogOpen: boolean
  isTranscriptExporting: boolean
  openReportDialog: () => void
}

export function useConversationReportActions({
  onGenerated,
  reportContext,
}: ConversationReportActionsInput): ConversationReportActionsState {
  const { t } = useTranslation()
  const [reportDialogOpen, setReportDialogOpen] = useState(false)
  const [generatedReportExport, setGeneratedReportExport] =
    useState<GenerateConversationReportResponse["export"] | null>(null)
  const transcriptExportMutation = useMutation({
    mutationFn: () =>
      exportConversationTranscriptApi(reportContext.conversationId!, {
        format: "markdown",
        includeFiles: true,
        includeRuntimeEvents: true,
        scope: "conversation_all",
      }),
    onSuccess: (response) => {
      downloadConversationTranscript(response)
      notifySuccess(t("agentOutput.backend.reportActions.transcriptExported"))
    },
    onError: () => {
      notifyError(t("agentOutput.backend.reportActions.transcriptFailed"))
    },
  })
  const generateReportMutation = useMutation({
    mutationFn: (draft: ConversationReportDraft) =>
      generateConversationReportApi(
        reportContext.conversationId!,
        createConversationReportRequest(draft, reportContext)
      ),
    onSuccess: (response) => {
      setGeneratedReportExport(response.export)
      setReportDialogOpen(false)
      notifySuccess(t("agentOutput.backend.reportActions.reportGenerated"))
      onGenerated(response)
    },
    onError: () => {
      notifyError(t("agentOutput.backend.reportActions.reportFailed"))
    },
  })

  return {
    artifactExport: null,
    canExportTranscript: Boolean(reportContext.conversationId),
    canGenerateReport: Boolean(reportContext.conversationId),
    closeReportDialog: () => setReportDialogOpen(false),
    exportTranscript: () => {
      if (!reportContext.conversationId || transcriptExportMutation.isPending) {
        return
      }
      transcriptExportMutation.mutate()
    },
    generatedReportExport,
    generateReport: (draft) => {
      if (!reportContext.conversationId || generateReportMutation.isPending) {
        return
      }
      generateReportMutation.mutate(draft)
    },
    isGeneratingReport: generateReportMutation.isPending,
    isReportDialogOpen: reportDialogOpen,
    isTranscriptExporting: transcriptExportMutation.isPending,
    openReportDialog: () => {
      if (!reportContext.conversationId) return
      setReportDialogOpen(true)
    },
  }
}

export function buildGeneratedReportArtifactExport(input: {
  generatedReportExport: GenerateConversationReportResponse["export"] | null
  selectedArtifact: WorkspaceArtifactResponse | null
  selectedArtifactId: string | null
}): WorkspaceArtifactExportResponse | null {
  const { generatedReportExport, selectedArtifact, selectedArtifactId } = input
  if (
    !generatedReportExport ||
    selectedArtifact?.id !== selectedArtifactId ||
    selectedArtifact?.source !== "manual_report_generation"
  ) {
    return null
  }

  return {
    artifactId: selectedArtifact.id,
    content: generatedReportExport.content,
    contentChars: generatedReportExport.contentChars,
    fileName: generatedReportExport.fileName,
    format: generatedReportExport.format,
    metadata: selectedArtifact.metadata,
    mimeType: generatedReportExport.mimeType,
    sourceFormat: selectedArtifact.format,
    sourceStorageKind: selectedArtifact.storageKind,
    title: selectedArtifact.title,
    warnings: generatedReportExport.warnings,
    workspaceId: selectedArtifact.workspaceId,
  }
}

function downloadConversationTranscript(response: ConversationTranscriptExportResponse) {
  downloadTextFile({
    content: response.content,
    fileName: response.fileName,
    mimeType: response.mimeType,
  })
}
