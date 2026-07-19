import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import {
  DEFAULT_RUN_TASK_VALUES,
  DEFAULT_RUN_VALUES,
  DEFAULT_TASK_VALUES,
  DEFAULT_WORKSPACE_VALUES,
  executionFields,
  runFields,
  runTaskFields,
  taskFields,
  workspaceFields,
} from "./workspace-dialog-fields"
import type { WorkspacesPageState } from "../use-workspaces-page"
import { useWorkspaceDialogOptions } from "../view-model/use-workspace-dialog-options"

export function WorkspaceDialogs({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()
  const isWorkspaceDialog =
    page.dialog === "create-workspace" || page.dialog === "edit-workspace"
  const { campaigns, products } = useWorkspaceDialogOptions(isWorkspaceDialog)

  return (
    <>
      {page.dialog === "create-workspace" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.workspace")}
          fields={workspaceFields(t, {
            campaigns,
            includeWorkspaceType: true,
            products,
          })}
          initialValues={DEFAULT_WORKSPACE_VALUES}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createWorkspace}
        />
      ) : null}
      {page.dialog === "edit-workspace" && page.editingWorkspace ? (
        <ControlledDialog
          key={page.editingWorkspace.id}
          title={t("commerce.workspaces.dialogs.editWorkspace")}
          fields={workspaceFields(t, {
            campaigns,
            includeWorkspaceType: false,
            products,
            selectedCampaignId: page.editingWorkspace.campaignId ?? "",
            selectedProductId: page.editingWorkspace.productId ?? "",
          })}
          initialValues={{
            campaignId: page.editingWorkspace.campaignId ?? "",
            productId: page.editingWorkspace.productId ?? "",
            title: page.editingWorkspace.title,
          }}
          notice={t("commerce.workspaces.dialogs.workspaceTypeLocked")}
          onClose={() => {
            page.setDialog(null)
            page.setEditingWorkspace(null)
          }}
          onSubmit={(values) =>
            page.updateWorkspace({
              values,
              workspaceId: page.editingWorkspace!.id,
            })
          }
        />
      ) : null}
      {page.dialog === "create-task" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.task")}
          fields={taskFields(t)}
          initialValues={DEFAULT_TASK_VALUES}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createTask}
        />
      ) : null}
      {page.dialog === "create-run" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.run")}
          fields={runFields(t)}
          initialValues={DEFAULT_RUN_VALUES}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createRun}
        />
      ) : null}
      {page.dialog === "create-run-task" ? (
        <ControlledDialog
          title={t("commerce.workspaces.dialogs.runTask")}
          fields={runTaskFields(t)}
          initialValues={DEFAULT_RUN_TASK_VALUES}
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
  notice,
  onClose,
  onSubmit,
  title,
}: {
  fields: SimpleField[]
  initialValues: SimpleFormValues
  notice?: string
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
      notice={notice}
      setValues={setValues}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) onClose()
      }}
      onSubmit={() => onSubmit(values)}
    />
  )
}
