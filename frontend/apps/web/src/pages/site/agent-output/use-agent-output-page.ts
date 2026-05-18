import {
  agentOutputActionKeys,
  agentOutputMetricKeys,
  agentOutputSectionKeys,
  agentOutputStepKeys,
} from "./agent-output-data"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useQuery } from "@tanstack/react-query"
import {
  exportWorkspaceArtifactApi,
  getWorkspaceArtifactApi,
  listWorkspaceArtifactsApi,
  listWorkspacesApi,
  type WorkspaceArtifactExportResponse,
  type WorkspaceArtifactResponse,
  type WorkspaceResponse,
} from "@/api"

export interface AgentOutputDeliverable extends WorkspaceArtifactResponse {
  workspace: WorkspaceResponse
}

export interface AgentOutputBackendState {
  artifactExport: WorkspaceArtifactExportResponse | null
  deliverables: AgentOutputDeliverable[]
  errorMessage: string | null
  isExportLoading: boolean
  loading: boolean
  refetch: () => void
  selectedArtifact: AgentOutputDeliverable | null
  selectedArtifactId: string | null
  setSelectedArtifactId: (artifactId: string) => void
  workspaces: WorkspaceResponse[]
}

export function useAgentOutputPage() {
  const { t } = useTranslation()
  const [selectedArtifactId, setSelectedArtifactId] = useState<string | null>(null)
  const backendQuery = useQuery({
    queryKey: ["agent-output", "backend-deliverables"],
    queryFn: loadBackendDeliverables,
  })
  const deliverables = useMemo(
    () => backendQuery.data?.deliverables ?? [],
    [backendQuery.data?.deliverables]
  )
  const selectedArtifactSnapshot = useMemo(() => {
    if (!deliverables.length) return null

    return (
      deliverables.find((artifact) => artifact.id === selectedArtifactId) ??
      deliverables[0]
    )
  }, [deliverables, selectedArtifactId])
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
      getWorkspaceArtifactApi(selectedWorkspace!.id, selectedArtifactSnapshot!.id),
  })
  const selectedArtifact = useMemo<AgentOutputDeliverable | null>(() => {
    if (!selectedArtifactSnapshot) return null
    if (!artifactDetailQuery.data) return selectedArtifactSnapshot

    return {
      ...artifactDetailQuery.data,
      workspace: selectedArtifactSnapshot.workspace,
    }
  }, [artifactDetailQuery.data, selectedArtifactSnapshot])
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
      exportWorkspaceArtifactApi(selectedArtifact!.workspaceId, selectedArtifact!.id, {
        format: selectedArtifact?.format,
      }),
  })
  const backendError =
    backendQuery.error ?? artifactDetailQuery.error ?? artifactExportQuery.error

  return {
    actions: agentOutputActionKeys.map((key) =>
      t(`agentOutput.actions.${key}`)
    ),
    backend: {
      artifactExport: artifactExportQuery.data ?? null,
      deliverables,
      errorMessage:
        backendError instanceof Error ? backendError.message : null,
      isExportLoading: artifactExportQuery.isLoading,
      loading: backendQuery.isLoading || artifactDetailQuery.isLoading,
      refetch: () => {
        void backendQuery.refetch()
      },
      selectedArtifact,
      selectedArtifactId: selectedArtifact?.id ?? selectedArtifactId,
      setSelectedArtifactId,
      workspaces: backendQuery.data?.workspaces ?? [],
    } satisfies AgentOutputBackendState,
    metrics: agentOutputMetricKeys.map((key) => ({
      label: t(`agentOutput.metrics.${key}.label`),
      value: t(`agentOutput.metrics.${key}.value`),
      description: t(`agentOutput.metrics.${key}.description`),
    })),
    prompt: t("agentOutput.prompt"),
    sections: agentOutputSectionKeys.map((key) => ({
      id: key,
      title: t(`agentOutput.sections.${key}.title`),
      eyebrow: t(`agentOutput.sections.${key}.eyebrow`),
      content: t(`agentOutput.sections.${key}.content`),
      bullets: [0, 1, 2].map((index) =>
        t(`agentOutput.sections.${key}.bullets.${index}`)
      ),
    })),
    steps: agentOutputStepKeys.map((key) => ({
      title: t(`agentOutput.steps.${key}.title`),
      owner: t(`agentOutput.steps.${key}.owner`),
      timing: t(`agentOutput.steps.${key}.timing`),
      detail: t(`agentOutput.steps.${key}.detail`),
    })),
  }
}

async function loadBackendDeliverables() {
  const workspaceResponse = await listWorkspacesApi({
    pageSize: 50,
    status: "active",
  })
  const workspaces = workspaceResponse.workspaces
  const artifactGroups = await Promise.all(
    workspaces.map(async (workspace) => {
      const artifactResponse = await listWorkspaceArtifactsApi(workspace.id, {
        pageSize: 20,
        status: "active",
      })

      return artifactResponse.artifacts.map((artifact) => ({
        ...artifact,
        workspace,
      }))
    })
  )
  const deliverables = artifactGroups.flat().sort(compareDeliverables)

  return {
    deliverables,
    workspaces,
  }
}

function compareDeliverables(
  left: AgentOutputDeliverable,
  right: AgentOutputDeliverable
) {
  return readDeliverableTime(right) - readDeliverableTime(left)
}

function readDeliverableTime(artifact: WorkspaceArtifactResponse) {
  return Math.max(readTime(artifact.updatedAt), readTime(artifact.createdAt))
}

function readTime(value?: string) {
  if (!value) return 0
  const parsed = Date.parse(value)
  return Number.isFinite(parsed) ? parsed : 0
}
