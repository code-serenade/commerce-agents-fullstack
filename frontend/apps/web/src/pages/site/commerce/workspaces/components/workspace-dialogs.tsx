import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { WorkspacesPageState } from "../use-workspaces-page"

export function WorkspaceDialogs({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()

  return (
    <>
      {page.dialog === "create-workspace" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.workspace")}
          fields={workspaceFields(t)}
          initialValues={{ workspaceType: "general" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createWorkspace}
        />
      ) : null}
      {page.dialog === "create-task" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.task")}
          fields={taskFields(t)}
          initialValues={{ priority: "0", status: "pending" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createTask}
        />
      ) : null}
      {page.dialog === "create-run" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.run")}
          fields={runFields(t)}
          initialValues={{ source: "manual", status: "planned" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createRun}
        />
      ) : null}
      {page.dialog === "create-run-task" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.runTask")}
          fields={runTaskFields(t)}
          initialValues={{
            acceptanceScore: "0",
            attempts: "0",
            seq: "1",
            status: "pending",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createRunTask}
        />
      ) : null}
      {page.dialog === "create-execution" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.execution")}
          fields={executionFields(t)}
          initialValues={{}}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createExecution}
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

function workspaceFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "title",
      label: t("commerce.fields.workspaceTitle"),
      required: true,
    },
    { key: "workspaceType", label: t("commerce.fields.workspaceType") },
    { key: "productId", label: t("commerce.fields.productId") },
    { key: "campaignId", label: t("commerce.fields.campaignId") },
  ]
}

function taskFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "title", label: t("commerce.fields.title"), required: true },
    { key: "ownerRole", label: t("commerce.fields.ownerRole") },
    { key: "status", label: t("commerce.common.status") },
    { key: "priority", label: t("commerce.fields.priority"), type: "number" },
    {
      key: "acceptanceCriteria",
      label: t("commerce.fields.acceptanceCriteria"),
      type: "textarea",
    },
    {
      key: "description",
      label: t("commerce.fields.description"),
      type: "textarea",
    },
  ]
}

function runFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "goal", label: t("commerce.fields.goal"), required: true },
    { key: "status", label: t("commerce.common.status") },
    { key: "source", label: t("commerce.common.source") },
    {
      key: "requirements",
      label: t("commerce.fields.requirements"),
      type: "textarea",
    },
    { key: "summary", label: t("commerce.fields.summary"), type: "textarea" },
  ]
}

function runTaskFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "workspaceTaskId", label: t("commerce.fields.workspaceTaskId") },
    { key: "seq", label: t("commerce.fields.seq"), type: "number" },
    { key: "ownerRole", label: t("commerce.fields.role") },
    { key: "status", label: t("commerce.common.status") },
    { key: "attempts", label: t("commerce.fields.attempts"), type: "number" },
    {
      key: "acceptanceScore",
      label: t("commerce.fields.acceptanceScore"),
      type: "number",
    },
    { key: "acceptancePassed", label: t("commerce.fields.acceptancePassed") },
    {
      key: "acceptanceNotes",
      label: t("commerce.fields.acceptanceNotes"),
      type: "textarea",
    },
    {
      key: "executionNotes",
      label: t("commerce.fields.executionNotes"),
      type: "textarea",
    },
  ]
}

function executionFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "workspaceTaskId", label: t("commerce.fields.workspaceTaskId") },
    {
      key: "actionType",
      label: t("commerce.fields.actionType"),
      required: true,
    },
    { key: "platform", label: t("commerce.common.platform") },
    { key: "expectedImpact", label: t("commerce.fields.expectedImpact") },
    {
      key: "description",
      label: t("commerce.fields.suggestionDescription"),
      required: true,
      type: "textarea",
    },
  ]
}
