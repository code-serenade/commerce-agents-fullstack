import type { ReactNode } from "react"
import { Toggle as CoreToggle } from "@workspace/ui-core/components/toggle"

export interface ToggleProps {
  pressed: boolean
  onPressedChange: (pressed: boolean) => void
  label: ReactNode
  disabled?: boolean
}

export function Toggle({
  pressed,
  onPressedChange,
  label,
  disabled = false,
}: ToggleProps) {
  return (
    <CoreToggle
      pressed={pressed}
      onPressedChange={onPressedChange}
      disabled={disabled}
    >
      {label}
    </CoreToggle>
  )
}
