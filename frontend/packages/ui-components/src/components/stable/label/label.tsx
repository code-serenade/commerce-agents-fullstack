import type { ReactNode } from "react"
import { Label as CoreLabel } from "@workspace/ui-core/components/label"

export interface LabelProps {
  children: ReactNode
}

export function Label({ children }: LabelProps) {
  return <CoreLabel>{children}</CoreLabel>
}
