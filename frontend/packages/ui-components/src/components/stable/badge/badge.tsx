import type { ReactNode } from "react"
import { Badge as CoreBadge } from "@workspace/ui-core/components/badge"

export type BadgeVariant =
  | "default"
  | "secondary"
  | "destructive"
  | "outline"
  | "ghost"
  | "link"

export interface BadgeProps {
  children: ReactNode
  variant?: BadgeVariant
}

export function Badge({ children, variant = "default" }: BadgeProps) {
  return <CoreBadge variant={variant}>{children}</CoreBadge>
}
