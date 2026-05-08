import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
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
