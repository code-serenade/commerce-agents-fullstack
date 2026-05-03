import type { ReactNode } from "react"
import {
  AlertDialog as CoreAlertDialog,
  AlertDialogAction as CoreAlertDialogAction,
  AlertDialogCancel as CoreAlertDialogCancel,
  AlertDialogContent as CoreAlertDialogContent,
  AlertDialogDescription as CoreAlertDialogDescription,
  AlertDialogFooter as CoreAlertDialogFooter,
  AlertDialogHeader as CoreAlertDialogHeader,
  AlertDialogTitle as CoreAlertDialogTitle,
  AlertDialogTrigger as CoreAlertDialogTrigger,
} from "@workspace/ui-core/components/alert-dialog"

export interface AlertDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  triggerLabel: ReactNode
  title: ReactNode
  description: ReactNode
  cancelLabel?: ReactNode
  confirmLabel?: ReactNode
  onConfirm?: () => void
}

export function AlertDialog({
  open,
  onOpenChange,
  triggerLabel,
  title,
  description,
  cancelLabel = "Cancel",
  confirmLabel = "Continue",
  onConfirm,
}: AlertDialogProps) {
  return (
    <CoreAlertDialog open={open} onOpenChange={onOpenChange}>
      <CoreAlertDialogTrigger>{triggerLabel}</CoreAlertDialogTrigger>
      <CoreAlertDialogContent>
        <CoreAlertDialogHeader>
          <CoreAlertDialogTitle>{title}</CoreAlertDialogTitle>
          <CoreAlertDialogDescription>{description}</CoreAlertDialogDescription>
        </CoreAlertDialogHeader>
        <CoreAlertDialogFooter>
          <CoreAlertDialogCancel>{cancelLabel}</CoreAlertDialogCancel>
          <CoreAlertDialogAction onClick={onConfirm}>
            {confirmLabel}
          </CoreAlertDialogAction>
        </CoreAlertDialogFooter>
      </CoreAlertDialogContent>
    </CoreAlertDialog>
  )
}
