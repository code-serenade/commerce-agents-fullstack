import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import type { WorkspaceResponse } from "@/api"
import { useWorkspaceActions } from "./use-workspace-actions"
import { buildWorkspaceMetricCards } from "./workspace-metrics"
import { useWorkspaceQueries } from "./use-workspace-queries"

export function useWorkspacesPage({
  workspaceId: routeWorkspaceId = null,
}: {
  workspaceId?: string | null
} = {}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [keyword, setKeyword] = useState("")
  const [localWorkspaceId, setLocalWorkspaceId] = useState<string | null>(null)
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null)
  const [selectedArtifactId, setSelectedArtifactId] = useState<string | null>(
    null
  )
  const [detailDrawer, setDetailDrawer] =
    useState<WorkspaceDetailDrawerName | null>(null)
  const [dialog, setDialog] = useState<WorkspaceDialogName | null>(null)
  const [editingWorkspace, setEditingWorkspace] =
    useState<WorkspaceResponse | null>(null)
  const selectedWorkspaceId = routeWorkspaceId ?? localWorkspaceId

  const setSelectedWorkspaceId = (workspaceId: string | null) => {
    setLocalWorkspaceId(workspaceId)
    setSelectedRunId(null)
    setSelectedArtifactId(null)
    setDetailDrawer(null)
    void navigate(
      workspaceId
        ? `/workspaces/${encodeURIComponent(workspaceId)}`
        : "/workspaces"
    )
  }

  const openWorkspaceDetail = (workspaceId: string) => {
    setSelectedWorkspaceId(workspaceId)
  }

  const openWorkspaceEdit = (workspace: WorkspaceResponse) => {
    setEditingWorkspace(workspace)
    setDialog("edit-workspace")
  }

  const closeWorkspaceDetail = () => {
    setSelectedWorkspaceId(null)
  }

  const closeDetailDrawer = () => {
    setDetailDrawer(null)
  }

  const openArtifactDetailDrawer = (artifactId: string) => {
    setSelectedArtifactId(artifactId)
    setDetailDrawer("artifact")
  }

  const openRunDetailDrawer = (runId: string) => {
    setSelectedRunId(runId)
    setDetailDrawer("run")
  }

  const queries = useWorkspaceQueries({
    keyword,
    selectedArtifactId,
    selectedRunId,
    selectedWorkspaceId,
  })
  const actions = useWorkspaceActions({
    selectedRunId: queries.selectedRun?.id,
    setDialog,
    setEditingWorkspace,
    setSelectedRunId,
    setSelectedWorkspaceId,
    t,
    workspaceId: queries.selectedWorkspace?.id,
  })
  const metricCards = useMemo(
    () =>
      buildWorkspaceMetricCards(t, {
        executions: queries.executionQueue,
        artifacts: queries.artifacts,
        runs: queries.runs,
        tasks: queries.tasks,
        workspaces: queries.workspaces,
      }),
    [
      queries.artifacts,
      queries.executionQueue,
      queries.runs,
      queries.tasks,
      queries.workspaces,
      t,
    ]
  )

  return {
    ...actions,
    ...queries,
    dialog,
    detailDrawer,
    closeDetailDrawer,
    closeWorkspaceDetail,
    editingWorkspace,
    isWorkspaceDetailOpen: Boolean(selectedWorkspaceId),
    keyword,
    metricCards,
    openArtifactDetailDrawer,
    openRunDetailDrawer,
    openWorkspaceDetail,
    openWorkspaceEdit,
    setSelectedArtifactId,
    setDetailDrawer,
    setDialog,
    setEditingWorkspace,
    setKeyword,
    setSelectedRunId,
    setSelectedWorkspaceId,
  }
}

export type WorkspaceDialogName =
  | "create-workspace"
  | "edit-workspace"
  | "create-task"
  | "create-run"
  | "create-run-task"
  | "create-execution"

export type WorkspaceDetailDrawerName = "run" | "artifact"

export type WorkspacesPageState = ReturnType<typeof useWorkspacesPage>
