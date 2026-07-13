import { useMutation, useQueryClient } from "@tanstack/react-query"
import {
  archiveWorkspaceApi,
  archiveWorkspaceTaskApi,
  createWorkspaceApi,
  createWorkspaceRunApi,
  createWorkspaceRunTaskApi,
  createWorkspaceTaskApi,
  updateWorkspaceApi,
  updateWorkspaceRunApi,
  updateWorkspaceRunTaskApi,
  updateWorkspaceTaskApi,
  type WorkspaceRunResponse,
  type WorkspaceRunTaskResponse,
  type WorkspaceTaskResponse,
} from "@/api"
import {
  notifySuccess,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import { DEFAULT_WORKSPACE_TYPE } from "../domain/workspace-types"
import type { WorkspaceDialogName } from "./use-workspaces-page"
import { workspacesQueryKey } from "./use-workspace-queries"
import { createRunTaskPayload, createTaskPayload } from "./workspace-payloads"
import { useWorkspaceExecutionActions } from "./use-workspace-execution-actions"

export function useWorkspaceActions({
  selectedRunId,
  setDialog,
  setEditingWorkspace,
  setSelectedRunId,
  setSelectedWorkspaceId,
  t,
  workspaceId,
}: {
  selectedRunId: string | undefined
  setDialog: (dialog: WorkspaceDialogName | null) => void
  setEditingWorkspace: (workspace: null) => void
  setSelectedRunId: (id: string | null) => void
  setSelectedWorkspaceId: (id: string | null) => void
  t: CommerceTranslate
  workspaceId: string | undefined
}) {
  const queryClient = useQueryClient()
  const invalidateWorkspaces = async () => {
    await queryClient.invalidateQueries({ queryKey: workspacesQueryKey })
  }
  const createWorkspace = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createWorkspaceApi({
        campaignId: toOptionalString(values.campaignId),
        config: {},
        productId: toOptionalString(values.productId),
        title: values.title,
        workspaceType: values.workspaceType || DEFAULT_WORKSPACE_TYPE,
      }),
    onSuccess: async (workspace) => {
      setSelectedWorkspaceId(workspace.id)
      setDialog(null)
      notifySuccess(t("commerce.workspaces.notify.workspaceCreated"))
      await invalidateWorkspaces()
    },
  })
  const archiveWorkspace = useMutation({
    mutationFn: archiveWorkspaceApi,
    onSuccess: async (_data, archivedWorkspaceId) => {
      if (workspaceId === archivedWorkspaceId) {
        setSelectedWorkspaceId(null)
      }
      notifySuccess(t("commerce.workspaces.notify.workspaceArchived"))
      await invalidateWorkspaces()
    },
  })
  const updateWorkspace = useMutation({
    mutationFn: ({
      values,
      workspaceId,
    }: {
      values: SimpleFormValues
      workspaceId: string
    }) =>
      updateWorkspaceApi(workspaceId, {
        campaignId: toOptionalString(values.campaignId),
        productId: toOptionalString(values.productId),
        title: values.title,
      }),
    onSuccess: async () => {
      setDialog(null)
      setEditingWorkspace(null)
      notifySuccess(t("commerce.workspaces.notify.workspaceUpdated"))
      await invalidateWorkspaces()
    },
  })
  const createTask = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createWorkspaceTaskApi(workspaceId!, createTaskPayload(values)),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.workspaces.notify.taskCreated"))
      await invalidateWorkspaces()
    },
  })
  const updateTaskStatus = useMutation({
    mutationFn: ({
      status,
      task,
    }: {
      task: WorkspaceTaskResponse
      status: string
    }) =>
      updateWorkspaceTaskApi(workspaceId!, task.id, {
        metadata: task.metadata ?? {},
        status,
        title: task.title,
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.workspaces.notify.taskUpdated"))
      await invalidateWorkspaces()
    },
  })
  const archiveTask = useMutation({
    mutationFn: (taskId: string) =>
      archiveWorkspaceTaskApi(workspaceId!, taskId),
    onSuccess: async () => {
      notifySuccess(t("commerce.workspaces.notify.taskArchived"))
      await invalidateWorkspaces()
    },
  })
  const createRun = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createWorkspaceRunApi(workspaceId!, {
        goal: values.goal,
        metadata: {},
        report: {},
        requirements: values.requirements
          ? values.requirements.split("\n").filter(Boolean)
          : [],
        source: values.source || "manual",
        status: values.status || "planned",
        summary: toOptionalString(values.summary),
      }),
    onSuccess: async (run) => {
      setSelectedRunId(run.id)
      setDialog(null)
      notifySuccess(t("commerce.workspaces.notify.runCreated"))
      await invalidateWorkspaces()
    },
  })
  const updateRunStatus = useMutation({
    mutationFn: ({
      run,
      status,
    }: {
      run: WorkspaceRunResponse
      status: string
    }) =>
      updateWorkspaceRunApi(workspaceId!, run.id, {
        goal: run.goal,
        metadata: run.metadata ?? {},
        status,
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.workspaces.notify.runUpdated"))
      await invalidateWorkspaces()
    },
  })
  const createRunTask = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createWorkspaceRunTaskApi(
        workspaceId!,
        selectedRunId!,
        createRunTaskPayload(values)
      ),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.workspaces.notify.runTaskCreated"))
      await invalidateWorkspaces()
    },
  })
  const updateRunTaskStatus = useMutation({
    mutationFn: ({
      runTask,
      status,
    }: {
      runTask: WorkspaceRunTaskResponse
      status: string
    }) =>
      updateWorkspaceRunTaskApi(workspaceId!, selectedRunId!, runTask.id, {
        metadata: runTask.metadata ?? {},
        ownerRole: runTask.ownerRole,
        seq: runTask.seq,
        status,
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.workspaces.notify.runTaskUpdated"))
      await invalidateWorkspaces()
    },
  })
  const { approveExecution, createExecution, rejectExecution } =
    useWorkspaceExecutionActions({
      invalidateWorkspaces,
      selectedRunId,
      setDialog,
      t,
      workspaceId,
    })

  return {
    approveExecution,
    archiveTask: archiveTask.mutate,
    archiveWorkspace: archiveWorkspace.mutate,
    createExecution,
    createRun: createRun.mutate,
    createRunTask: createRunTask.mutate,
    createTask: createTask.mutate,
    createWorkspace: createWorkspace.mutate,
    rejectExecution,
    updateWorkspace: updateWorkspace.mutate,
    updateRunStatus: updateRunStatus.mutate,
    updateRunTaskStatus: updateRunTaskStatus.mutate,
    updateTaskStatus: updateTaskStatus.mutate,
  }
}
