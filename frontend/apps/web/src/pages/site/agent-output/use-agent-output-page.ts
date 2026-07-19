import { useMemo, useState } from "react"
import { useSearchParams } from "react-router"
import { useQuery } from "@tanstack/react-query"
import {
  exportWorkspaceArtifactApi,
  getWorkspaceArtifactApi,
  type WorkspaceArtifactExportResponse,
  type WorkspaceResponse,
} from "@/api"
import {
  loadBackendDeliverables,
  type AgentOutputDeliverable,
} from "./agent-output-backend-data"
import {
  getConversationReportContext,
  isConversationReportArtifact,
  type ConversationReportDraft,
} from "./view-model/conversation-report-view-model"
import {
  buildGeneratedReportArtifactExport,
  useConversationReportActions,
} from "./view-model/use-conversation-report-actions"

export type { AgentOutputDeliverable } from "./agent-output-backend-data"

export interface AgentOutputBackendState {
  artifactExport: WorkspaceArtifactExportResponse | null
  canExportTranscript: boolean
  canGenerateReport: boolean
  closeReportDialog: () => void
  context: ReturnType<typeof getConversationReportContext>
  currentConversationArtifactId: string | null
  deliverables: AgentOutputDeliverable[]
  errorMessage: string | null
  exportTranscript: () => void
  generateReport: (draft: ConversationReportDraft) => void
  isExportLoading: boolean
  isGeneratingReport: boolean
  isReportDialogOpen: boolean
  isTranscriptExporting: boolean
  loading: boolean
  openReportDialog: () => void
  refetch: () => void
  selectedArtifact: AgentOutputDeliverable | null
  selectedArtifactId: string | null
  setSelectedArtifactId: (artifactId: string) => void
  workspaces: WorkspaceResponse[]
}

export function useAgentOutputPage() {
  const [searchParams] = useSearchParams()
  const [selectedArtifactId, setSelectedArtifactId] = useState<string | null>(
    null
  )
  const backendQuery = useQuery({
    queryKey: ["agent-output", "backend-deliverables"],
    queryFn: loadBackendDeliverables,
  })
  const deliverables = useMemo(
    () => backendQuery.data?.deliverables ?? [],
    [backendQuery.data?.deliverables]
  )
  const requestedConversationId =
    searchParams.get("conversationId")?.trim() || null
  const requestedConversationArtifact = useMemo(() => {
    if (!requestedConversationId) return null

    return (
      deliverables.find((artifact) =>
        isConversationReportArtifact(artifact, requestedConversationId)
      ) ?? null
    )
  }, [deliverables, requestedConversationId])
  const selectedArtifactSnapshot = useMemo(() => {
    if (!deliverables.length) return null
    if (requestedConversationId && !requestedConversationArtifact) {
      return (
        deliverables.find((artifact) => artifact.id === selectedArtifactId) ??
        null
      )
    }

    return (
      deliverables.find((artifact) => artifact.id === selectedArtifactId) ??
      requestedConversationArtifact ??
      deliverables[0]
    )
  }, [
    deliverables,
    requestedConversationArtifact,
    requestedConversationId,
    selectedArtifactId,
  ])
  const selectedWorkspace = selectedArtifactSnapshot?.workspace
  const artifactDetailQuery = useQuery({
    queryKey: [
      "agent-output",
      "backend-deliverables",
      selectedWorkspace?.id,
      selectedArtifactSnapshot?.id,
      "detail",
    ],
    enabled: Boolean(selectedWorkspace?.id && selectedArtifactSnapshot?.id),
    queryFn: () =>
      getWorkspaceArtifactApi(
        selectedWorkspace!.id,
        selectedArtifactSnapshot!.id
      ),
  })
  const selectedArtifact = useMemo<AgentOutputDeliverable | null>(() => {
    if (!selectedArtifactSnapshot) return null
    if (!artifactDetailQuery.data) return selectedArtifactSnapshot

    return {
      ...artifactDetailQuery.data,
      workspace: selectedArtifactSnapshot.workspace,
    }
  }, [artifactDetailQuery.data, selectedArtifactSnapshot])
  const reportContext = useMemo(
    () => getConversationReportContext(searchParams, selectedArtifact),
    [searchParams, selectedArtifact]
  )
  const artifactExportQuery = useQuery({
    queryKey: [
      "agent-output",
      "backend-deliverables",
      selectedArtifact?.workspaceId,
      selectedArtifact?.id,
      "export",
      selectedArtifact?.format,
    ],
    enabled: Boolean(selectedArtifact?.workspaceId && selectedArtifact?.id),
    queryFn: () =>
      exportWorkspaceArtifactApi(
        selectedArtifact!.workspaceId,
        selectedArtifact!.id,
        {
          format: selectedArtifact?.format,
        }
      ),
  })
  const reportActions = useConversationReportActions({
    onGenerated: (response) => {
      setSelectedArtifactId(response.artifact.id)
      void backendQuery.refetch()
    },
    reportContext,
  })
  const backendError =
    backendQuery.error ?? artifactDetailQuery.error ?? artifactExportQuery.error
  const artifactExport = useMemo(() => {
    return (
      buildGeneratedReportArtifactExport({
        generatedReportExport: reportActions.generatedReportExport,
        selectedArtifact,
        selectedArtifactId,
      }) ??
      artifactExportQuery.data ??
      null
    )
  }, [
    artifactExportQuery.data,
    reportActions.generatedReportExport,
    selectedArtifact,
    selectedArtifactId,
  ])

  return {
    backend: {
      artifactExport,
      canExportTranscript: reportActions.canExportTranscript,
      canGenerateReport: reportActions.canGenerateReport,
      closeReportDialog: reportActions.closeReportDialog,
      context: reportContext,
      currentConversationArtifactId: requestedConversationArtifact?.id ?? null,
      deliverables,
      errorMessage: backendError instanceof Error ? backendError.message : null,
      exportTranscript: reportActions.exportTranscript,
      generateReport: reportActions.generateReport,
      isExportLoading: artifactExportQuery.isLoading,
      isGeneratingReport: reportActions.isGeneratingReport,
      isReportDialogOpen: reportActions.isReportDialogOpen,
      isTranscriptExporting: reportActions.isTranscriptExporting,
      loading: backendQuery.isLoading || artifactDetailQuery.isLoading,
      openReportDialog: reportActions.openReportDialog,
      refetch: () => {
        void backendQuery.refetch()
      },
      selectedArtifact,
      selectedArtifactId: selectedArtifact?.id ?? selectedArtifactId,
      setSelectedArtifactId,
      workspaces: backendQuery.data?.workspaces ?? [],
    } satisfies AgentOutputBackendState,
  }
}
