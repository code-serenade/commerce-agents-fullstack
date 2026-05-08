import { useState, type ChangeEvent } from "react"
import { useTranslation } from "react-i18next"
import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
} from "@workspace/ui-components"
import {
  SimpleFormDialog,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { ProductsPageState } from "../use-products-page"
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
      {page.dialog === "create-asset" ? <AssetDialog page={page} /> : null}
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

function AssetDialog({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()
  const [values, setValues] = useState<SimpleFormValues>({
    assetType: "docs",
    fileName: "",
    fileSize: "",
    mimeType: "",
    ["description"]: "",
  })
  const updateValue = (key: string, value: string) => {
    setValues((current) => ({ ...current, [key]: value }))
  }
  const handleFile = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0] ?? null
    page.setAssetFile(file)
    if (!file) return
    setValues((current) => ({
      ...current,
      fileName: current.fileName || file.name,
      fileSize: String(file.size),
      mimeType: file.type || current.mimeType,
    }))
  }

  return (
    <Dialog
      open={page.dialog === "create-asset"}
      onOpenChange={(open) => {
        if (!open) {
          page.setAssetFile(null)
          page.setDialog(null)
        }
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("commerce.products.dialogs.asset")}</DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <label className="block space-y-1.5">
            <span className="text-sm font-medium">
              {t("commerce.products.dialogs.uploadFile")}
            </span>
            <input
              type="file"
              className="block w-full text-sm text-muted-foreground file:mr-3 file:rounded-md file:border-0 file:bg-foreground file:px-3 file:py-2 file:text-sm file:text-background"
              onChange={handleFile}
            />
          </label>
          {assetFields(t)
            .filter((field) => field.key !== "objectKey")
            .map((field) => (
            <label key={field.key} className="block space-y-1.5">
              <span className="text-sm font-medium">{field.label}</span>
              <Input
                value={values[field.key] ?? ""}
                placeholder={field.placeholder}
                onValueChange={(value) => updateValue(field.key, value)}
              />
            </label>
          ))}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => page.setDialog(null)}>
            {t("commerce.common.cancel")}
          </Button>
          <Button
            disabled={!page.assetFile || !values.fileName.trim()}
            onClick={() => page.createAsset(values)}
          >
            {t("commerce.common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
