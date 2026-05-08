import { useMutation } from "@tanstack/react-query"
import {
  approveExecutionQueueApi,
  createExecutionQueueApi,
  rejectExecutionQueueApi,
  type ExecutionQueueResponse,
} from "@/api"
import {
  notifySuccess,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import type { WorkspaceDialogName } from "./use-workspaces-page"

export function useWorkspaceExecutionActions({
  invalidateWorkspaces,
  selectedRunId,
  setDialog,
  t,
  workspaceId,
}: {
  invalidateWorkspaces: () => Promise<void>
  selectedRunId: string | undefined
  setDialog: (dialog: WorkspaceDialogName | null) => void
  t: CommerceTranslate
  workspaceId: string | undefined
}) {
  const createExecution = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createExecutionQueueApi({
        actionType: values.actionType,
        description: values.description,
        expectedImpact: toOptionalString(values.expectedImpact),
        payload: {},
        platform: toOptionalString(values.platform),
        runId: selectedRunId ?? toOptionalString(values.runId),
        workspaceId: workspaceId ?? toOptionalString(values.workspaceId),
        workspaceTaskId: toOptionalString(values.workspaceTaskId),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.workspaces.notify.executionCreated"))
      await invalidateWorkspaces()
    },
  })
  const approveExecution = useMutation({
    mutationFn: (execution: ExecutionQueueResponse) =>
      approveExecutionQueueApi(execution.id, {
        reason: t("commerce.common.manualApprove"),
        result: {},
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.workspaces.notify.executionApproved"))
      await invalidateWorkspaces()
    },
  })
  const rejectExecution = useMutation({
    mutationFn: (execution: ExecutionQueueResponse) =>
      rejectExecutionQueueApi(execution.id, {
        reason: t("commerce.common.manualReject"),
        result: {},
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.workspaces.notify.executionRejected"))
      await invalidateWorkspaces()
    },
  })

  return {
    approveExecution: approveExecution.mutate,
    createExecution: createExecution.mutate,
    rejectExecution: rejectExecution.mutate,
  }
}
