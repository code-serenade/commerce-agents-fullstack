import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { ApprovalsPageState } from "../use-approvals-page"

export function ApprovalDialogs({ page }: { page: ApprovalsPageState }) {
  const { t } = useTranslation()

  return (
    <>
      {page.dialog === "approval" ? (
        <ControlledDialog
          title={t("commerce.approvals.dialogs.approval")}
          fields={approvalFields(t)}
          initialValues={{ riskLevel: "L1" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createApproval}
        />
      ) : null}
      {page.dialog === "audit" ? (
        <ControlledDialog
          title={t("commerce.approvals.dialogs.audit")}
          fields={auditFields(t)}
          initialValues={{ status: "success" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createAudit}
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

function approvalFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "skillName", label: t("commerce.fields.skillName"), required: true },
    { key: "riskLevel", label: t("commerce.fields.riskLevel") },
    { key: "workspaceId", label: t("commerce.fields.workspaceId") },
    { key: "runId", label: t("commerce.fields.runId") },
    { key: "conversationId", label: t("commerce.fields.conversationId") },
    { key: "requestRole", label: t("commerce.fields.requestRole") },
    { key: "toolCallId", label: t("commerce.fields.toolCallId") },
    { key: "actionKey", label: t("commerce.fields.actionKey") },
    { key: "reason", label: t("commerce.common.reason"), type: "textarea" },
  ]
}

function auditFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "action", label: t("commerce.fields.action"), required: true },
    { key: "targetType", label: t("commerce.fields.targetType"), required: true },
    { key: "targetId", label: t("commerce.fields.targetId") },
    { key: "workspaceId", label: t("commerce.fields.workspaceId") },
    { key: "runId", label: t("commerce.fields.runId") },
    { key: "executionQueueId", label: t("commerce.fields.executionQueueId") },
    { key: "toolApprovalId", label: t("commerce.fields.toolApprovalId") },
    { key: "status", label: t("commerce.common.status") },
    { key: "requestId", label: t("commerce.fields.requestId") },
    { key: "traceId", label: t("commerce.fields.traceId") },
    { key: "reason", label: t("commerce.common.reason"), type: "textarea" },
  ]
}
