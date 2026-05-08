import { useMutation, useQueryClient } from "@tanstack/react-query"
import {
  actAlertApi,
  archiveDailyBriefingApi,
  cancelLongTaskApi,
  consumeAllBackgroundEventsApi,
  consumeBackgroundEventApi,
  createAlertApi,
  createBackgroundEventApi,
  createLongTaskApi,
  dismissAlertApi,
  dismissAllAlertsApi,
  saveDailyBriefingApi,
  snoozeAlertApi,
  updateLongTaskApi,
  type AlertResponse,
  type BackgroundEventResponse,
  type DailyBriefingResponse,
  type LongTaskResponse,
} from "@/api"
import {
  notifySuccess,
  todayDateString,
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import type { OperationDialogName } from "./use-operations-page"
import { operationsQueryKey } from "./use-operation-queries"

export function useOperationActions({
  setDialog,
  t,
}: {
  setDialog: (dialog: OperationDialogName | null) => void
  t: CommerceTranslate
}) {
  const queryClient = useQueryClient()
  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey: operationsQueryKey })
  }
  const createLongTask = useMutation({
    mutationFn: (values: SimpleFormValues) => createLongTaskApi(longTaskPayload(values)),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.operations.notify.longTaskCreated"))
      await invalidate()
    },
  })
  const updateLongTaskStatus = useMutation({
    mutationFn: ({ status, task }: { task: LongTaskResponse; status: string }) =>
      updateLongTaskApi(task.id, {
        currentStep: task.currentStep,
        errorMessage: task.errorMessage,
        progress: task.progress,
        result: task.result ?? {},
        role: task.role,
        status,
        title: task.title,
        totalSteps: task.totalSteps,
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.longTaskUpdated"))
      await invalidate()
    },
  })
  const cancelLongTask = useMutation({
    mutationFn: (task: LongTaskResponse) =>
      cancelLongTaskApi(task.id, {
        errorMessage: t("commerce.common.manualCancel"),
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.longTaskCancelled"))
      await invalidate()
    },
  })
  const saveDailyBriefing = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      saveDailyBriefingApi({
        alertSummary: {},
        archived: false,
        briefingDate: values.briefingDate || todayDateString(),
        productSummary: {},
        sections: [],
        source: values.source || "manual",
        status: values.status || "ready",
        summary: values.summary,
        taskSummary: {},
        title: values.title,
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.operations.notify.briefingSaved"))
      await invalidate()
    },
  })
  const archiveDailyBriefing = useMutation({
    mutationFn: (record: DailyBriefingResponse) => archiveDailyBriefingApi(record.id),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.briefingArchived"))
      await invalidate()
    },
  })
  const createNotification = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createBackgroundEventApi({
        consumed: values.consumed === "true",
        conversationId: toOptionalString(values.conversationId),
        data: {},
        eventType: values.eventType,
        title: values.title,
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.operations.notify.notificationCreated"))
      await invalidate()
    },
  })
  const consumeEvent = useMutation({
    mutationFn: (event: BackgroundEventResponse) =>
      consumeBackgroundEventApi(event.id),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.notificationRead"))
      await invalidate()
    },
  })
  const consumeAllEvents = useMutation({
    mutationFn: () => consumeAllBackgroundEventsApi({}),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.allRead"))
      await invalidate()
    },
  })
  const createAlert = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      createAlertApi({
        alertType: values.alertType,
        dedupKey: toOptionalString(values.dedupKey),
        message: values.message,
        payload: {},
        role: toOptionalString(values.role),
        runId: toOptionalString(values.runId),
        severity: values.severity || "warning",
        snoozedUntil: toOptionalString(values.snoozedUntil),
        workspaceId: toOptionalString(values.workspaceId),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.operations.notify.alertCreated"))
      await invalidate()
    },
  })
  const dismissAlert = useMutation({
    mutationFn: (alert: AlertResponse) =>
      dismissAlertApi(alert.id, { reason: t("commerce.common.manualClose") }),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.alertClosed"))
      await invalidate()
    },
  })
  const actAlert = useMutation({
    mutationFn: (alert: AlertResponse) => actAlertApi(alert.id, { result: {} }),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.alertActed"))
      await invalidate()
    },
  })
  const snoozeAlert = useMutation({
    mutationFn: (alert: AlertResponse) =>
      snoozeAlertApi(alert.id, {
        snoozedUntil: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.alertSnoozed"))
      await invalidate()
    },
  })
  const dismissAllAlerts = useMutation({
    mutationFn: () => dismissAllAlertsApi({}),
    onSuccess: async () => {
      notifySuccess(t("commerce.operations.notify.alertsClosed"))
      await invalidate()
    },
  })

  return {
    actAlert: actAlert.mutate,
    archiveDailyBriefing: archiveDailyBriefing.mutate,
    cancelLongTask: cancelLongTask.mutate,
    consumeAllEvents: consumeAllEvents.mutate,
    consumeEvent: consumeEvent.mutate,
    createAlert: createAlert.mutate,
    createLongTask: createLongTask.mutate,
    createNotification: createNotification.mutate,
    dismissAlert: dismissAlert.mutate,
    dismissAllAlerts: dismissAllAlerts.mutate,
    saveDailyBriefing: saveDailyBriefing.mutate,
    snoozeAlert: snoozeAlert.mutate,
    updateLongTaskStatus: updateLongTaskStatus.mutate,
  }
}

function longTaskPayload(values: SimpleFormValues) {
  return {
    currentStep: toOptionalString(values.currentStep),
    errorMessage: toOptionalString(values.errorMessage),
    inputParams: {},
    progress: toNumberOrNull(values.progress) ?? 0,
    result: {},
    role: toOptionalString(values.role),
    runId: toOptionalString(values.runId),
    status: values.status || "pending",
    taskType: values.taskType || "manual",
    title: values.title,
    totalSteps: toNumberOrNull(values.totalSteps) ?? 100,
    workspaceId: toOptionalString(values.workspaceId),
  }
}
