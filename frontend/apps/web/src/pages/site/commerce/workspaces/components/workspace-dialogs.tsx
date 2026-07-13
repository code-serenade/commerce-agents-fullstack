import { useState } from "react"
import { useTranslation } from "react-i18next"
import { useQuery } from "@tanstack/react-query"
import {
  listCampaignsApi,
  listProductsApi,
  type CampaignResponse,
  type ProductResponse,
} from "@/api"
import {
  SimpleFormDialog,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import {
  workspaceRunStatusOptions,
  workspaceRunTaskStatusOptions,
  workspaceTaskStatusOptions,
} from "../../domain/status-options"
import {
  DEFAULT_WORKSPACE_TYPE,
  workspaceTypeOptions,
} from "../../domain/workspace-types"
import type { WorkspacesPageState } from "../use-workspaces-page"

export function WorkspaceDialogs({ page }: { page: WorkspacesPageState }) {
  const { t } = useTranslation()
  const isWorkspaceDialog =
    page.dialog === "create-workspace" || page.dialog === "edit-workspace"
  const productsQuery = useQuery({
    enabled: isWorkspaceDialog,
    queryKey: ["commerce", "workspaces", "binding-products"],
    queryFn: () => listProductsApi({ pageSize: 100 }),
  })
  const campaignsQuery = useQuery({
    enabled: isWorkspaceDialog,
    queryKey: ["commerce", "workspaces", "binding-campaigns"],
    queryFn: () => listCampaignsApi({ pageSize: 100 }),
  })
  const products = productsQuery.data ?? []
  const campaigns = campaignsQuery.data?.campaigns ?? []

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
          initialValues={{ workspaceType: DEFAULT_WORKSPACE_TYPE }}
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

function workspaceFields(
  t: CommerceTranslate,
  {
    campaigns,
    includeWorkspaceType,
    products,
    selectedCampaignId = "",
    selectedProductId = "",
  }: {
    campaigns: CampaignResponse[]
    includeWorkspaceType: boolean
    products: ProductResponse[]
    selectedCampaignId?: string
    selectedProductId?: string
  }
): SimpleField[] {
  const fields: SimpleField[] = [
    {
      key: "title",
      label: t("commerce.fields.workspaceTitle"),
      required: true,
    },
  ]

  if (includeWorkspaceType) {
    fields.push({
      key: "workspaceType",
      label: t("commerce.fields.workspaceType"),
      options: workspaceTypeOptions(t),
      type: "select",
    })
  }

  fields.push(
    {
      key: "productId",
      label: t("commerce.workspaces.dialogs.productBinding"),
      options: productOptions(t, products, selectedProductId),
      type: "select",
    },
    {
      key: "campaignId",
      label: t("commerce.workspaces.dialogs.campaignBinding"),
      options: campaignOptions(t, campaigns, selectedCampaignId),
      type: "select",
    }
  )

  return fields
}

function productOptions(
  t: CommerceTranslate,
  products: ProductResponse[],
  selectedProductId: string
) {
  const options = [
    { label: t("commerce.common.none"), value: "" },
    ...products.map((product) => ({
      label: compactLabel([
        product.name,
        product.platform,
        product.externalId,
        product.id,
      ]),
      value: product.id,
    })),
  ]
  return appendMissingSelectedOption(
    options,
    selectedProductId,
    t("commerce.workspaces.dialogs.productBinding")
  )
}

function campaignOptions(
  t: CommerceTranslate,
  campaigns: CampaignResponse[],
  selectedCampaignId: string
) {
  const options = [
    { label: t("commerce.common.none"), value: "" },
    ...campaigns.map((campaign) => ({
      label: compactLabel([
        campaign.name,
        campaign.campaignType,
        campaign.status,
        campaign.id,
      ]),
      value: campaign.id,
    })),
  ]
  return appendMissingSelectedOption(
    options,
    selectedCampaignId,
    t("commerce.workspaces.dialogs.campaignBinding")
  )
}

function appendMissingSelectedOption(
  options: Array<{ label: string; value: string }>,
  selectedValue: string,
  fallbackLabel: string
) {
  if (!selectedValue || options.some((option) => option.value === selectedValue)) {
    return options
  }
  return [
    ...options,
    {
      label: `${fallbackLabel}: ${selectedValue}`,
      value: selectedValue,
    },
  ]
}

function compactLabel(parts: Array<string | null | undefined>) {
  const [primary, ...rest] = parts
  const suffix = rest.filter(Boolean).join(" / ")
  return suffix ? `${primary} (${suffix})` : (primary ?? "-")
}

function taskFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "title", label: t("commerce.fields.title"), required: true },
    { key: "ownerRole", label: t("commerce.fields.ownerRole") },
    {
      key: "status",
      label: t("commerce.common.status"),
      options: workspaceTaskStatusOptions(t),
      type: "select",
    },
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
    {
      key: "status",
      label: t("commerce.common.status"),
      options: workspaceRunStatusOptions(t),
      type: "select",
    },
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
    {
      key: "status",
      label: t("commerce.common.status"),
      options: workspaceRunTaskStatusOptions(t),
      type: "select",
    },
    { key: "attempts", label: t("commerce.fields.attempts"), type: "number" },
    {
      key: "acceptanceScore",
      label: t("commerce.fields.acceptanceScore"),
      type: "number",
    },
    {
      key: "acceptancePassed",
      label: t("commerce.fields.acceptancePassed"),
      options: booleanOptions,
      type: "select",
    },
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

const booleanOptions = [
  { label: String("false"), value: "false" },
  { label: String("true"), value: "true" },
]
