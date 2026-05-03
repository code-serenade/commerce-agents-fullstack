import type { ReactNode } from "react"
import { TriangleAlert } from "lucide-react"
import {
  Alert as CoreAlert,
  AlertDescription as CoreAlertDescription,
  AlertTitle as CoreAlertTitle,
} from "@workspace/ui-core/components/alert"

export type AppNoticeVariant = "warning"

export interface AppNoticeProps {
  description: ReactNode
  title: ReactNode
  variant?: AppNoticeVariant
}

export function AppNotice({
  description,
  title,
  variant = "warning",
}: AppNoticeProps) {
  const icon = variant === "warning" ? <TriangleAlert className="size-4" /> : null

  return (
    <CoreAlert
      variant={variant}
      className="inline-grid w-fit max-w-full border-[color-mix(in_oklab,var(--warning)_35%,var(--border))] text-(--app-text) *:[svg]:text-(--warning)"
    >
      {icon}
      <CoreAlertTitle className="text-sm font-semibold">{title}</CoreAlertTitle>
      <CoreAlertDescription className="text-(--app-muted-text)">
        {description}
      </CoreAlertDescription>
    </CoreAlert>
  )
}
