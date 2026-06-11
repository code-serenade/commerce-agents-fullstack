import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
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
import type { ProductsPageState } from "../use-products-page"
import {
  productDraftValues,
  productSampleValues,
} from "../product-form-payloads"
import { ProductAssetDialog } from "./product-asset-dialog"
import {
  assetInitialValues,
  assetFields,
  changeFields,
  competitorInitialValues,
  competitorFields,
  knowledgeInitialValues,
  knowledgeFields,
  materialInitialValues,
  materialFields,
  productFields,
  productInitialValues,
} from "./product-dialog-fields"

export function ProductDialogs({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()

  return (
    <>
      {page.dialog === "create-product" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.create")}
          fields={productFields(t)}
          initialValues={{ currency: "CNY" }}
          isSubmitting={page.createProductPending}
          notice={t("commerce.common.smartFillReviewNotice")}
          sampleAction={{
            label: t("commerce.common.fillSample"),
            values: productSampleValues(t),
          }}
          smartFillAction={{
            disabled: page.createProductDraftPending,
            label: t("commerce.common.templateSmartFill"),
            onClick: async (values) => {
              const result = await page.createProductDraft({ values })
              return {
                agentContext: result.agentContext,
                generationMode: result.generationMode,
                values: productDraftValues(result.draft),
              }
            },
          }}
          agentSmartFillAction={{
            disabled: page.createProductDraftPending,
            label: t("commerce.common.agentSmartFill"),
            onClick: async (values, onProcess) => {
              const result = await page.createProductDraftStream({
                onProcess: (event) =>
                  onProcess?.(processEventDraftResult(event)),
                values,
              })
              return {
                agentContext: result.agentContext,
                generationMode: result.generationMode,
                stages: ["form_draft_output_validation_passed"],
                values: productDraftValues(result.draft),
              }
            },
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createProduct}
        />
      ) : null}
      {page.dialog === "edit-product" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.edit")}
          fields={productFields(t)}
          initialValues={productInitialValues(page)}
          onClose={() => page.setDialog(null)}
          onSubmit={(values) => {
            if (!page.selectedProduct) return
            page.updateProduct({ target: page.selectedProduct, values })
          }}
        />
      ) : null}
      {page.dialog === "create-knowledge" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.knowledge")}
          fields={knowledgeFields(t)}
          initialValues={{
            confidence: "0.8",
            contentType: "selling_point",
            sourceType: "manual",
          }}
          isSubmitting={page.createKnowledgePending}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createKnowledge}
        />
      ) : null}
      {page.dialog === "edit-knowledge" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.editKnowledge")}
          fields={knowledgeFields(t)}
          initialValues={knowledgeInitialValues(page)}
          onClose={() => page.setDialog(null)}
          onSubmit={(values) => {
            if (!page.selectedKnowledge) return
            page.updateKnowledge({ target: page.selectedKnowledge, values })
          }}
        />
      ) : null}
      {page.dialog === "create-asset" ? (
        <ProductAssetDialog page={page} />
      ) : null}
      {page.dialog === "edit-asset" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.editAsset")}
          fields={assetFields(t).filter((field) => field.key !== "objectKey")}
          initialValues={assetInitialValues(page)}
          onClose={() => page.setDialog(null)}
          onSubmit={(values) => {
            if (!page.selectedAsset) return
            page.updateAsset({ target: page.selectedAsset, values })
          }}
        />
      ) : null}
      {page.dialog === "create-material" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.material")}
          fields={materialFields(t)}
          initialValues={{ materialType: "copy" }}
          isSubmitting={page.createMaterialPending}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createMaterial}
        />
      ) : null}
      {page.dialog === "edit-material" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.editMaterial")}
          fields={materialFields(t).filter((field) => field.key !== "materialType")}
          initialValues={materialInitialValues(page)}
          onClose={() => page.setDialog(null)}
          onSubmit={(values) => {
            if (!page.selectedMaterial) return
            page.updateMaterial({ target: page.selectedMaterial, values })
          }}
        />
      ) : null}
      {page.dialog === "create-competitor" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.competitor")}
          fields={competitorFields(t)}
          initialValues={{ platform: page.selectedProduct?.platform ?? "" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createCompetitor}
        />
      ) : null}
      {page.dialog === "edit-competitor" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.editCompetitor")}
          fields={competitorFields(t)}
          initialValues={competitorInitialValues(page)}
          onClose={() => page.setDialog(null)}
          onSubmit={(values) => {
            if (!page.selectedCompetitor) return
            page.updateCompetitor({ target: page.selectedCompetitor, values })
          }}
        />
      ) : null}
      {page.dialog === "create-competitor-change" ? (
        <ControlledSimpleDialog
          title={t("commerce.products.dialogs.change")}
          fields={changeFields(t)}
          initialValues={{ changeType: "price", sourceType: "manual" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.createCompetitorChange}
        />
      ) : null}
    </>
  )
}

function ControlledSimpleDialog({
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
  fields: Parameters<typeof SimpleFormDialog>[0]["fields"]
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
