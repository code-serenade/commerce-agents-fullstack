import type { ReactNode } from "react"
import {
  Collapsible as CoreCollapsible,
  CollapsibleContent as CoreCollapsibleContent,
  CollapsibleTrigger as CoreCollapsibleTrigger,
} from "@workspace/ui-core/components/collapsible"

export interface CollapsibleProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  triggerLabel: ReactNode
  disabled?: boolean
  children: ReactNode
}

export function Collapsible({
  open,
  onOpenChange,
  triggerLabel,
  disabled = false,
  children,
}: CollapsibleProps) {
  return (
    <CoreCollapsible open={open} onOpenChange={onOpenChange} disabled={disabled}>
      <CoreCollapsibleTrigger disabled={disabled}>{triggerLabel}</CoreCollapsibleTrigger>
      <CoreCollapsibleContent>{children}</CoreCollapsibleContent>
    </CoreCollapsible>
  )
}
