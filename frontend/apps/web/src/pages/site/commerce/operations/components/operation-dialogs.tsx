import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  todayDateString,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { OperationsPageState } from "../use-operations-page"

export function OperationDialogs({ page }: { page: OperationsPageState }) {
  const { t } = useTranslation()

  return (
    <>
      {page.dialog === "long-task" ? (
        <ControlledDialog
          title={t("commerce.operations.dialogs.longTask")}
          fields={longTaskFields(t)}
          initialValues={{
            progress: "0",
            status: "pending",
            taskType: "manual",
            totalSteps: "100",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createLongTask}
        />
      ) : null}
      {page.dialog === "daily-briefing" ? (
        <ControlledDialog
          title={t("commerce.operations.dialogs.briefing")}
          fields={dailyFields(t)}
          initialValues={{
            briefingDate: todayDateString(),
            source: "manual",
            status: "ready",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.saveDailyBriefing}
        />
      ) : null}
      {page.dialog === "notification" ? (
        <ControlledDialog
          title={t("commerce.operations.dialogs.notification")}
          fields={notificationFields(t)}
          initialValues={{ consumed: "false", eventType: "task.completed" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createNotification}
        />
      ) : null}
      {page.dialog === "alert" ? (
        <ControlledDialog
          title={t("commerce.operations.dialogs.alert")}
          fields={alertFields(t)}
          initialValues={{ alertType: "policy_risk", severity: "warning" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createAlert}
        />
      ) : null}
    </>
  )
}

function ControlledDialog({
  fields,
  initialValues,
  onClose,
  onSubmit,
  title,
}: {
  fields: SimpleField[]
  initialValues: SimpleFormValues
  onClose: () => void
  onSubmit: (values: SimpleFormValues) => void
  title: string
}) {
  const [values, setValues] = useState<SimpleFormValues>(initialValues)

  return (
    <SimpleFormDialog
      open
      title={title}
      fields={fields}
      values={values}
      setValues={setValues}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) onClose()
      }}
      onSubmit={() => onSubmit(values)}
    />
  )
}

function longTaskFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "title", label: t("commerce.fields.title"), required: true },
    { key: "taskType", label: t("commerce.fields.taskType") },
    { key: "workspaceId", label: t("commerce.fields.workspaceId") },
    { key: "runId", label: t("commerce.fields.runId") },
    { key: "status", label: t("commerce.common.status") },
    { key: "progress", label: t("commerce.fields.progress"), type: "number" },
    { key: "totalSteps", label: t("commerce.fields.totalSteps"), type: "number" },
    { key: "currentStep", label: t("commerce.fields.currentStep") },
    { key: "role", label: t("commerce.fields.role") },
    { key: "errorMessage", label: t("commerce.fields.errorMessage") },
  ]
}

function dailyFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "briefingDate", label: t("commerce.fields.briefingDate"), required: true },
    { key: "title", label: t("commerce.fields.title"), required: true },
    { key: "source", label: t("commerce.common.source") },
    { key: "status", label: t("commerce.common.status") },
    {
      key: "summary",
      label: t("commerce.fields.summary"),
      required: true,
      type: "textarea",
    },
  ]
}

function notificationFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "eventType", label: t("commerce.fields.eventType"), required: true },
    { key: "title", label: t("commerce.fields.title"), required: true },
    { key: "conversationId", label: t("commerce.fields.conversationId") },
    { key: "consumed", label: t("commerce.fields.consumed") },
  ]
}

function alertFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "alertType", label: t("commerce.fields.alertType"), required: true },
    { key: "severity", label: t("commerce.fields.severity") },
    { key: "workspaceId", label: t("commerce.fields.workspaceId") },
    { key: "runId", label: t("commerce.fields.runId") },
    { key: "role", label: t("commerce.fields.role") },
    { key: "dedupKey", label: t("commerce.fields.dedupKey") },
    { key: "snoozedUntil", label: t("commerce.fields.snoozedUntil") },
    {
      key: "message",
      label: t("commerce.fields.message"),
      required: true,
      type: "textarea",
    },
  ]
}
