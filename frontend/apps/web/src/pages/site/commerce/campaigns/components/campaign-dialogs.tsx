import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import {
  buildSimpleFormAuxiliaryActions,
  type SimpleFormDraftAction,
  type SimpleFormDraftResult,
  type SimpleFormSampleAction,
} from "../../components/commerce-form-actions"
import {
  mergeDraftProcessResult,
  processEventDraftResult,
} from "../../components/commerce-form-draft-state"
import { buildFormDraftProcessNotice } from "../../components/commerce-form-draft-process"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { CampaignsPageState } from "../use-campaigns-page"

export function CampaignDialogs({ page }: { page: CampaignsPageState }) {
  const { t } = useTranslation()

  return (
    <>
      {page.dialog === "create" ? (
        <CampaignDialog
          title={t("commerce.campaigns.dialogs.create")}
          fields={campaignFields(t)}
          initialValues={{
            campaignType: "promotion",
            status: "draft",
          }}
          isSubmitting={page.createCampaignPending}
          notice={t("commerce.common.smartFillReviewNotice")}
          sampleAction={{
            label: t("commerce.common.fillSample"),
            values: campaignSampleValues(t),
          }}
          smartFillAction={{
            disabled: page.createCampaignFormDraftPending,
            label: t("commerce.common.templateSmartFill"),
            onClick: async (values) => {
              const result = await page.createCampaignFormDraft({ values })
              return {
                agentContext: result.agentContext,
                generationMode: result.generationMode,
                values: campaignDraftValues(result.draft),
              }
            },
          }}
          agentSmartFillAction={{
            disabled: page.createCampaignFormDraftPending,
            label: t("commerce.common.agentSmartFill"),
            onClick: async (values, onProcess) => {
              const result = await page.createCampaignFormDraftStream({
                onProcess: (event) =>
                  onProcess?.(processEventDraftResult(event)),
                values,
              })
              return {
                agentContext: result.agentContext,
                generationMode: result.generationMode,
                stages: ["form_draft_output_validation_passed"],
                values: campaignDraftValues(result.draft),
              }
            },
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createCampaign}
        />
      ) : null}
      {page.dialog === "edit" ? (
        <CampaignDialog
          title={t("commerce.campaigns.dialogs.edit")}
          fields={campaignFields(t)}
          initialValues={{
            budget: page.editingCampaign?.budget
              ? String(page.editingCampaign.budget)
              : "",
            campaignType: page.editingCampaign?.campaignType ?? "promotion",
            description: page.editingCampaign?.description ?? "",
            endDate: page.editingCampaign?.endDate ?? "",
            name: page.editingCampaign?.name ?? "",
            productId: page.editingCampaign?.productId ?? "",
            startDate: page.editingCampaign?.startDate ?? "",
            status: page.editingCampaign?.status ?? "draft",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={(values) => {
            if (!page.editingCampaign) return
            page.updateCampaign({ campaign: page.editingCampaign, values })
          }}
        />
      ) : null}
    </>
  )
}

function CampaignDialog({
  fields,
  initialValues,
  isSubmitting,
  notice,
  onClose,
  onSubmit,
  sampleAction,
  agentSmartFillAction,
  smartFillAction,
  title,
}: {
  fields: SimpleField[]
  initialValues: SimpleFormValues
  isSubmitting?: boolean
  notice?: string
  onClose: () => void
  onSubmit: (values: SimpleFormValues) => void
  sampleAction?: SimpleFormSampleAction
  agentSmartFillAction?: SimpleFormDraftAction
  smartFillAction?: SimpleFormDraftAction
  title: string
}) {
  const { t } = useTranslation()
  const [draftResult, setDraftResult] = useState<SimpleFormDraftResult | null>(null)
  const [values, setValues] = useState<SimpleFormValues>(initialValues)
  const updateDraftResult = (result: SimpleFormDraftResult | null) => {
    setDraftResult((current) => mergeDraftProcessResult(current, result))
  }

  return (
    <SimpleFormDialog
      open
      title={title}
      auxiliaryActions={buildSimpleFormAuxiliaryActions({
        agentSmartFillAction,
        onDraftResult: updateDraftResult,
        sampleAction,
        setValues,
        smartFillAction,
        values,
      })}
      fields={fields}
      isSubmitting={isSubmitting}
      notice={notice}
      processNotice={buildFormDraftProcessNotice(draftResult, t)}
      values={values}
      setValues={setValues}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) onClose()
      }}
      onSubmit={() => onSubmit(values)}
    />
  )
}

function campaignDraftValues(
  draft: Awaited<
    ReturnType<CampaignsPageState["createCampaignFormDraft"]>
  >["draft"]
): SimpleFormValues {
  return {
    budget: draft.budget == null ? "" : String(draft.budget),
    campaignType: draft.campaignType ?? "",
    description: draft.description ?? "",
    endDate: draft.endDate ?? "",
    name: draft.name,
    productId: draft.productId ?? "",
    startDate: draft.startDate ?? "",
    status: draft.status,
  }
}

function campaignSampleValues(
  t: ReturnType<typeof useTranslation>["t"]
): SimpleFormValues {
  return {
    budget: "3000",
    campaignType: "promotion",
    description: t("commerce.campaigns.samples.campaign.description"),
    endDate: "2026-06-30",
    name: t("commerce.campaigns.samples.campaign.name"),
    productId: "",
    startDate: "2026-06-24",
    status: "draft",
  }
}

function campaignFields(t: CommerceTranslate): SimpleField[] {
  return [
    {
      key: "name",
      label: t("commerce.fields.campaignName"),
      required: true,
    },
    { key: "productId", label: t("commerce.fields.productId") },
    { key: "campaignType", label: t("commerce.fields.campaignType") },
    { key: "budget", label: t("commerce.fields.budget"), type: "number" },
    { key: "startDate", label: t("commerce.fields.startDate") },
    { key: "endDate", label: t("commerce.fields.endDate") },
    { key: "status", label: t("commerce.common.status") },
    {
      key: "description",
      label: t("commerce.fields.description"),
      type: "textarea",
    },
  ]
}
