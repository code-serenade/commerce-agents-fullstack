import { useState, type ChangeEvent } from "react"
import { useTranslation } from "react-i18next"
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
} from "@workspace/ui-components"
import type { SimpleFormValues } from "../../components/commerce-ui"
import type { ProductsPageState } from "../use-products-page"
import { assetFields } from "./product-dialog-fields"

export function ProductAssetDialog({ page }: { page: ProductsPageState }) {
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
          <DialogDescription>
            {t("commerce.common.dialogDescription")}
          </DialogDescription>
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
