import { useTranslation } from "react-i18next"
import type { ReactNode } from "react"
import type { CSSProperties } from "react"
import { ClipboardList, X } from "lucide-react"
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
import { normalizeNumericInput } from "./commerce-form-utils"

export interface SimpleField {
  decimalScale?: number
  key: string
  label: string
  layout?: "half"
  nonNegative?: boolean
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

const simpleFormDialogContentClassName =
  "flex max-h-[min(92vh,720px)] w-[min(var(--commerce-form-dialog-width),calc(100vw_-_2rem))] max-w-[calc(100vw_-_2rem)] flex-col gap-0 overflow-hidden rounded-2xl border border-(--document-card-border) bg-(image:--document-card-background) p-0 text-foreground shadow-[0_28px_80px_color-mix(in_oklch,var(--primary)_14%,transparent),0_1px_2px_color-mix(in_oklch,var(--foreground)_10%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_96%,transparent)] backdrop-blur-xl sm:max-w-[calc(100vw_-_2rem)]"

const simpleFormDialogBodyClassName =
  "grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-y-auto bg-[color-mix(in_oklch,var(--background)_88%,transparent)] px-5 py-4 [scrollbar-gutter:stable] sm:grid-cols-2 [&_[data-slot=input]]:h-10 [&_[data-slot=input]]:rounded-lg [&_[data-slot=input]]:border-(--document-card-border) [&_[data-slot=input]]:bg-[color-mix(in_oklch,var(--background)_92%,transparent)] [&_[data-slot=native-select-wrapper]]:h-10 [&_[data-slot=native-select-wrapper]]:w-full [&_[data-slot=native-select-wrapper]]:min-w-0 [&_[data-slot=native-select]]:h-10 [&_[data-slot=native-select]]:rounded-lg [&_[data-slot=native-select]]:border-(--document-card-border) [&_[data-slot=native-select]]:bg-[color-mix(in_oklch,var(--background)_92%,transparent)] [&_[data-slot=textarea]]:rounded-lg [&_[data-slot=textarea]]:border-(--document-card-border) [&_[data-slot=textarea]]:bg-[color-mix(in_oklch,var(--background)_92%,transparent)]"

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
  const resolvedDescription =
    description ?? t("commerce.common.dialogDescription")
  const updateValue = (field: SimpleField, value: string) => {
    setValues({
      ...values,
      [field.key]:
        field.type === "number"
          ? normalizeNumericInput(value, field)
          : value,
    })
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
        className={simpleFormDialogContentClassName}
        style={
          {
            "--commerce-form-dialog-width": dialogWidth,
          } as CSSProperties
        }
      >
        <DialogHeader className="relative shrink-0 gap-0 px-5 pt-5 pb-4">
          <div className="flex min-w-0 items-start justify-between gap-3">
            <div className="flex min-w-0 items-start gap-3">
              <span className="flex size-10 shrink-0 items-center justify-center rounded-xl border border-[color-mix(in_oklch,var(--primary)_18%,var(--document-card-border))] bg-[linear-gradient(135deg,color-mix(in_oklch,var(--app-active-bg)_88%,var(--background))_0%,color-mix(in_oklch,var(--background)_76%,var(--primary)_24%)_100%)] text-(--app-active-text) shadow-[0_10px_24px_color-mix(in_oklch,var(--primary)_10%,transparent)]">
                <ClipboardList className="size-5" />
              </span>
              <div className="min-w-0 space-y-1.5 pt-0.5">
                <DialogTitle>{title}</DialogTitle>
                <DialogDescription>{resolvedDescription}</DialogDescription>
              </div>
            </div>
            <DialogClose asChild>
              <Button
                aria-label={t("commerce.common.close")}
                className="-mt-2 -mr-2 rounded-lg hover:bg-(--shell-sidebar-item-hover-background)"
                size="icon"
                variant="ghost"
              >
                <X className="size-4" />
              </Button>
            </DialogClose>
          </div>
          <div className="mt-4 h-px w-full bg-[linear-gradient(90deg,var(--document-card-border)_0%,color-mix(in_oklch,var(--document-card-border)_48%,transparent)_58%,transparent_100%)]" />
        </DialogHeader>
        <div className={simpleFormDialogBodyClassName}>
          {notice ? (
            <div className="rounded-lg border border-dashed border-(--document-card-border) bg-[color-mix(in_oklch,var(--app-active-bg)_34%,transparent)] px-3 py-2 text-xs/5 text-muted-foreground sm:col-span-2">
              {notice}
            </div>
          ) : null}
          {processNotice ? (
            <div className="rounded-lg border border-(--document-card-border) bg-[color-mix(in_oklch,var(--background)_92%,transparent)] px-3 py-2 text-xs/5 text-muted-foreground sm:col-span-2">
              {processNotice}
            </div>
          ) : null}
          {fields.map((field) => (
            <label
              key={field.key}
              className={`block min-w-0 space-y-2 ${
                field.layout === "half" ? "" : "sm:col-span-2"
              }`}
            >
              <span className="flex items-center gap-1.5 text-sm font-semibold text-foreground">
                {field.label}
                {field.required ? (
                  <span className="text-destructive" aria-hidden="true">
                    *
                  </span>
                ) : null}
              </span>
              {field.type === "textarea" ? (
                <Textarea
                  value={values[field.key] ?? ""}
                  rows={4}
                  placeholder={field.placeholder}
                  onValueChange={(value) => updateValue(field, value)}
                />
              ) : field.type === "select" ? (
                <NativeSelect
                  value={values[field.key] ?? ""}
                  placeholder={field.placeholder}
                  options={field.options ?? []}
                  onValueChange={(value) => updateValue(field, value)}
                />
              ) : (
                <Input
                  value={values[field.key] ?? ""}
                  type={field.type === "number" ? "number" : "text"}
                  placeholder={field.placeholder}
                  onValueChange={(value) => updateValue(field, value)}
                />
              )}
            </label>
          ))}
        </div>
        <DialogFooter className="m-0 flex shrink-0 flex-row flex-wrap justify-center gap-2 rounded-none rounded-b-2xl border-t border-(--document-card-border) bg-[linear-gradient(180deg,color-mix(in_oklch,var(--background)_92%,transparent)_0%,color-mix(in_oklch,var(--background)_84%,var(--primary)_4%)_100%)] px-5 py-4 min-[769px]:justify-end">
          {footerAuxiliaryActions.map((action) => (
            <Button
              key={action.label}
              className="h-9 min-w-20 cursor-pointer rounded-lg"
              variant="outline"
              disabled={isSubmitting || action.disabled}
              onClick={action.onClick}
            >
              {action.label}
            </Button>
          ))}
          <Button
            className="h-9 min-w-20 cursor-pointer rounded-lg"
            variant="outline"
            disabled={isSubmitting}
            onClick={() => onOpenChange(false)}
          >
            {t("commerce.common.cancel")}
          </Button>
          <Button
            className="h-9 min-w-20 cursor-pointer rounded-lg disabled:cursor-not-allowed disabled:border-(--document-card-border) disabled:bg-muted/45 disabled:text-muted-foreground"
            disabled={missingRequired || isSubmitting}
            onClick={onSubmit}
          >
            {submitLabel ?? t("commerce.common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
