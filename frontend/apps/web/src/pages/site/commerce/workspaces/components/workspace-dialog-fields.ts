import type { CampaignResponse, ProductResponse } from "@/api"
import {
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import {
  commerceBooleanOptions,
  DEFAULT_WORKSPACE_TYPE,
  workspaceRunStatusOptions,
  workspaceRunTaskStatusOptions,
  workspaceTaskStatusOptions,
  workspaceTypeOptions,
} from "../../domain/enum-mappings"

export const DEFAULT_WORKSPACE_VALUES: SimpleFormValues = {
  workspaceType: DEFAULT_WORKSPACE_TYPE,
}

export const DEFAULT_TASK_VALUES: SimpleFormValues = {
  priority: "0",
  status: "pending",
}

export const DEFAULT_RUN_VALUES: SimpleFormValues = {
  source: "manual",
  status: "planned",
}

export const DEFAULT_RUN_TASK_VALUES: SimpleFormValues = {
  acceptanceScore: "0",
  attempts: "0",
  seq: "1",
  status: "pending",
}

export function workspaceFields(
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

export function taskFields(t: CommerceTranslate): SimpleField[] {
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

export function runFields(t: CommerceTranslate): SimpleField[] {
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

export function runTaskFields(t: CommerceTranslate): SimpleField[] {
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
      options: commerceBooleanOptions(t),
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

export function executionFields(t: CommerceTranslate): SimpleField[] {
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
