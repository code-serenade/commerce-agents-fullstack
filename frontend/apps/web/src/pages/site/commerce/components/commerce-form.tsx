import { useTranslation } from "react-i18next"
import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  NativeSelect,
  Textarea,
} from "@workspace/ui-components"
import type { NativeSelectOption } from "@workspace/ui-components"

export interface SimpleField {
  key: string
  label: string
  placeholder?: string
  type?: "text" | "number" | "date" | "textarea" | "select"
  options?: NativeSelectOption[]
  required?: boolean
}

export type SimpleFormValues = Record<string, string>

export function SimpleFormDialog({
  fields,
  onOpenChange,
  onSubmit,
  open,
  submitLabel,
  title,
  values,
  setValues,
}: {
  fields: SimpleField[]
  onOpenChange: (open: boolean) => void
  onSubmit: () => void
  open: boolean
  submitLabel?: string
  title: string
  values: SimpleFormValues
  setValues: (values: SimpleFormValues) => void
}) {
  const { t } = useTranslation()
  const updateValue = (key: string, value: string) => {
    setValues({ ...values, [key]: value })
  }
  const missingRequired = fields.some(
    (field) => field.required && !values[field.key]?.trim()
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          {fields.map((field) => (
            <label key={field.key} className="block space-y-1.5">
              <span className="text-sm font-medium text-foreground">
                {field.label}
              </span>
              {field.type === "textarea" ? (
                <Textarea
                  value={values[field.key] ?? ""}
                  rows={4}
                  placeholder={field.placeholder}
                  onValueChange={(value) => updateValue(field.key, value)}
                />
              ) : field.type === "select" ? (
                <NativeSelect
                  value={values[field.key] ?? ""}
                  placeholder={field.placeholder}
                  options={field.options ?? []}
                  onValueChange={(value) => updateValue(field.key, value)}
                />
              ) : (
                <Input
                  value={values[field.key] ?? ""}
                  type={field.type === "number" ? "number" : "text"}
                  placeholder={field.placeholder}
                  onValueChange={(value) => updateValue(field.key, value)}
                />
              )}
            </label>
          ))}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("commerce.common.cancel")}
          </Button>
          <Button disabled={missingRequired} onClick={onSubmit}>
            {submitLabel ?? t("commerce.common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
