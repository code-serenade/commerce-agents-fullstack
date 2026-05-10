import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  exportWorkspaceArtifactApi,
  getExecutionQueueApi,
  getWorkspaceArtifactApi,
  getWorkspaceApi,
  getWorkspaceRunApi,
  listExecutionQueueApi,
  listWorkspaceArtifactsApi,
  listWorkspaceRunsApi,
  listWorkspaceRunTasksApi,
  listWorkspacesApi,
  listWorkspaceTasksApi,
} from "@/api"

export const workspacesQueryKey = ["commerce", "workspaces"]

export function useWorkspaceQueries({
  keyword,
  selectedArtifactId,
  selectedRunId,
  selectedWorkspaceId,
}: {
  keyword: string
  selectedArtifactId: string | null
  selectedRunId: string | null
  selectedWorkspaceId: string | null
}) {
  const workspacesQuery = useQuery({
    queryKey: [...workspacesQueryKey, keyword],
    queryFn: () =>
      listWorkspacesApi({
        keyword: keyword || undefined,
        pageSize: 100,
        status: "active",
      }),
  })
  const workspaces = useMemo(
    () => workspacesQuery.data?.workspaces ?? [],
    [workspacesQuery.data?.workspaces]
  )
  const selectedWorkspaceSnapshot = useMemo(() => {
    if (!workspaces.length) return null

    return (
      workspaces.find((workspace) => workspace.id === selectedWorkspaceId) ??
      workspaces[0]
    )
  }, [selectedWorkspaceId, workspaces])
  const workspaceId = selectedWorkspaceSnapshot?.id
  const workspaceDetailQuery = useQuery({
    queryKey: [...workspacesQueryKey, workspaceId, "detail"],
    enabled: Boolean(workspaceId),
    queryFn: () => getWorkspaceApi(workspaceId!),
  })
  const selectedWorkspace =
    workspaceDetailQuery.data ?? selectedWorkspaceSnapshot
  const tasksQuery = useQuery({
    queryKey: [...workspacesQueryKey, workspaceId, "tasks"],
    enabled: Boolean(workspaceId),
    queryFn: () => listWorkspaceTasksApi(workspaceId!, { pageSize: 100 }),
  })
  const runsQuery = useQuery({
    queryKey: [...workspacesQueryKey, workspaceId, "runs"],
    enabled: Boolean(workspaceId),
    queryFn: () => listWorkspaceRunsApi(workspaceId!, { pageSize: 100 }),
  })
  const executionQueueQuery = useQuery({
    queryKey: [...workspacesQueryKey, workspaceId, "execution-queue"],
    queryFn: () =>
      listExecutionQueueApi({
        pageSize: 100,
        workspaceId: workspaceId || undefined,
      }),
  })
  const runs = useMemo(
    () => runsQuery.data?.runs ?? [],
    [runsQuery.data?.runs]
  )
  const selectedRunSnapshot = useMemo(() => {
    if (!runs.length) return null

    return runs.find((run) => run.id === selectedRunId) ?? runs[0]
  }, [runs, selectedRunId])
  const runDetailQuery = useQuery({
    queryKey: [...workspacesQueryKey, workspaceId, "runs", selectedRunSnapshot?.id, "detail"],
    enabled: Boolean(workspaceId && selectedRunSnapshot?.id),
    queryFn: () => getWorkspaceRunApi(workspaceId!, selectedRunSnapshot!.id),
  })
  const selectedRun = runDetailQuery.data ?? selectedRunSnapshot
  const runTasksQuery = useQuery({
    queryKey: [...workspacesQueryKey, workspaceId, "runs", selectedRun?.id, "tasks"],
    enabled: Boolean(workspaceId && selectedRun?.id),
    queryFn: () => listWorkspaceRunTasksApi(workspaceId!, selectedRun!.id),
  })
  const artifactsQuery = useQuery({
    queryKey: [...workspacesQueryKey, workspaceId, "artifacts"],
    enabled: Boolean(workspaceId),
    queryFn: () =>
      listWorkspaceArtifactsApi(workspaceId!, {
        pageSize: 100,
        status: "active",
      }),
  })
  const artifacts = useMemo(
    () => artifactsQuery.data?.artifacts ?? [],
    [artifactsQuery.data?.artifacts]
  )
  const selectedArtifactSnapshot = useMemo(() => {
    if (!artifacts.length) return null

    return (
      artifacts.find((artifact) => artifact.id === selectedArtifactId) ??
      artifacts[0]
    )
  }, [artifacts, selectedArtifactId])
  const artifactDetailQuery = useQuery({
    queryKey: [
      ...workspacesQueryKey,
      workspaceId,
      "artifacts",
      selectedArtifactSnapshot?.id,
      "detail",
    ],
    enabled: Boolean(workspaceId && selectedArtifactSnapshot?.id),
    queryFn: () =>
      getWorkspaceArtifactApi(workspaceId!, selectedArtifactSnapshot!.id),
  })
  const selectedArtifact =
    artifactDetailQuery.data ?? selectedArtifactSnapshot
  const artifactExportQuery = useQuery({
    queryKey: [
      ...workspacesQueryKey,
      workspaceId,
      "artifacts",
      selectedArtifact?.id,
      "export",
    ],
    enabled: Boolean(workspaceId && selectedArtifact?.id),
    queryFn: () =>
      exportWorkspaceArtifactApi(workspaceId!, selectedArtifact!.id, {
        format: selectedArtifact?.format,
      }),
  })
  const selectedExecutionSnapshot =
    executionQueueQuery.data?.items.find((item) => item.status === "pending") ??
    executionQueueQuery.data?.items[0] ??
    null
  const executionDetailQuery = useQuery({
    queryKey: [
      ...workspacesQueryKey,
      "execution-queue",
      selectedExecutionSnapshot?.id,
      "detail",
    ],
    enabled: Boolean(selectedExecutionSnapshot?.id),
    queryFn: () => getExecutionQueueApi(selectedExecutionSnapshot!.id),
  })

  return {
    artifacts,
    artifactExport: artifactExportQuery.data ?? null,
    executionQueue: executionQueueQuery.data?.items ?? [],
    focusedExecution:
      executionDetailQuery.data ?? selectedExecutionSnapshot,
    loading:
      workspacesQuery.isLoading ||
      workspaceDetailQuery.isLoading ||
      tasksQuery.isLoading ||
      runsQuery.isLoading ||
      runDetailQuery.isLoading ||
      executionQueueQuery.isLoading ||
      executionDetailQuery.isLoading ||
      runTasksQuery.isLoading ||
      artifactsQuery.isLoading ||
      artifactDetailQuery.isLoading ||
      artifactExportQuery.isLoading,
    runTasks: runTasksQuery.data ?? [],
    runs,
    selectedArtifact,
    selectedRun,
    selectedWorkspace,
    tasks: tasksQuery.data?.tasks ?? [],
    workspaces,
  }
}
