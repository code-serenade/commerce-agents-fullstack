import type { ReactNode } from "react"
import {
  HoverCard as CoreHoverCard,
  HoverCardContent as CoreHoverCardContent,
  HoverCardTrigger as CoreHoverCardTrigger,
} from "@workspace/ui-core/components/hover-card"

export interface HoverCardProps {
  triggerLabel: ReactNode
  title: ReactNode
  description?: ReactNode
  openDelay?: number
  closeDelay?: number
}

export function HoverCard({
  triggerLabel,
  title,
  description,
  openDelay = 300,
  closeDelay = 200,
}: HoverCardProps) {
  return (
    <CoreHoverCard openDelay={openDelay} closeDelay={closeDelay}>
      <CoreHoverCardTrigger>{triggerLabel}</CoreHoverCardTrigger>
      <CoreHoverCardContent>
        <div className="space-y-1">
          <div className="text-sm font-medium">{title}</div>
          {description ? (
            <div className="text-sm text-muted-foreground">{description}</div>
          ) : null}
        </div>
      </CoreHoverCardContent>
    </CoreHoverCard>
  )
}
