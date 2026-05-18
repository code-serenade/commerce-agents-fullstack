import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { ProductsPageState } from "../use-products-page"
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
  onClose,
  onSubmit,
  title,
}: {
  fields: Parameters<typeof SimpleFormDialog>[0]["fields"]
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
