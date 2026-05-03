import type { ReactNode } from "react"
import {
  Alert as CoreAlert,
  AlertDescription as CoreAlertDescription,
  AlertTitle as CoreAlertTitle,
} from "@workspace/ui-core/components/alert"

export type AlertVariant = "default" | "success" | "warning" | "error"

export type AlertProps = {
  variant?: AlertVariant
  title?: ReactNode
  description?: ReactNode
}

export function Alert({ variant = "default", title, description }: AlertProps) {
  return (
    <CoreAlert variant={variant}>
      {title && <CoreAlertTitle>{title}</CoreAlertTitle>}
      {description && (
        <CoreAlertDescription>{description}</CoreAlertDescription>
      )}
    </CoreAlert>
  )
}
