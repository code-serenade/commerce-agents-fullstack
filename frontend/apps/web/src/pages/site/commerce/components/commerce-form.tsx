import { useTranslation } from "react-i18next"
import type { ReactNode } from "react"
import type { CSSProperties } from "react"
import { X } from "lucide-react"
import {
  Button,
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
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

export interface SimpleFormAuxiliaryAction {
  label: string
  onClick: () => void | Promise<void>
  disabled?: boolean
}

export function SimpleFormDialog({
  auxiliaryAction,
  auxiliaryActions,
  fields,
  isSubmitting = false,
  notice,
  onOpenChange,
  onSubmit,
  open,
  processNotice,
  description,
  dialogWidth = "640px",
  submitLabel,
  title,
  values,
  setValues,
}: {
  auxiliaryAction?: SimpleFormAuxiliaryAction
  auxiliaryActions?: SimpleFormAuxiliaryAction[]
  fields: SimpleField[]
  isSubmitting?: boolean
  notice?: ReactNode
  onOpenChange: (open: boolean) => void
  onSubmit: () => void
  open: boolean
  processNotice?: ReactNode
  description?: ReactNode
  dialogWidth?: string
  submitLabel?: string
  title: string
  values: SimpleFormValues
  setValues: (values: SimpleFormValues) => void
}) {
  const { t } = useTranslation()
  const resolvedDescription = description ?? t("commerce.common.dialogDescription")
  const updateValue = (key: string, value: string) => {
    setValues({ ...values, [key]: value })
  }
  const missingRequired = fields.some(
    (field) => field.required && !values[field.key]?.trim()
  )
  const footerAuxiliaryActions = [
    ...(auxiliaryAction ? [auxiliaryAction] : []),
    ...(auxiliaryActions ?? []),
  ]

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        showCloseButton={false}
        className="flex max-h-[98vh] w-[min(var(--commerce-form-dialog-width),98vw)] max-w-[98vw] flex-col gap-0 overflow-hidden bg-white p-0 sm:max-w-[98vw] dark:bg-popover"
        style={
          {
            "--commerce-form-dialog-width": dialogWidth,
          } as CSSProperties
        }
      >
        <DialogHeader className="shrink-0 border-b border-border bg-white px-4 pt-4 pb-3 dark:bg-popover">
          <div className="flex min-w-0 items-start justify-between gap-3">
            <div className="min-w-0 space-y-2">
              <DialogTitle>{title}</DialogTitle>
              <DialogDescription>{resolvedDescription}</DialogDescription>
            </div>
            <DialogClose asChild>
              <Button
                aria-label={t("commerce.common.close")}
                className="-mt-2 -mr-2"
                size="icon"
                variant="ghost"
              >
                <X className="size-4" />
              </Button>
            </DialogClose>
          </div>
        </DialogHeader>
        <div className="min-h-0 flex-1 space-y-3 overflow-y-auto bg-white p-4 [scrollbar-gutter:stable] dark:bg-popover">
          {notice ? (
            <div className="rounded-md border border-dashed border-border bg-muted/30 px-3 py-2 text-xs/5 text-muted-foreground">
              {notice}
            </div>
          ) : null}
          {processNotice ? (
            <div className="rounded-md border border-border bg-background px-3 py-2 text-xs/5 text-muted-foreground">
              {processNotice}
            </div>
          ) : null}
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
        <DialogFooter className="m-0 flex shrink-0 flex-row flex-wrap justify-center gap-2 rounded-none rounded-b-xl border-t border-border bg-white p-4 min-[769px]:justify-end dark:bg-popover">
          {footerAuxiliaryActions.map((action) => (
            <Button
              key={action.label}
              variant="outline"
              disabled={isSubmitting || action.disabled}
              onClick={action.onClick}
            >
              {action.label}
            </Button>
          ))}
          <Button
            variant="outline"
            disabled={isSubmitting}
            onClick={() => onOpenChange(false)}
          >
            {t("commerce.common.cancel")}
          </Button>
          <Button disabled={missingRequired || isSubmitting} onClick={onSubmit}>
            {submitLabel ?? t("commerce.common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
