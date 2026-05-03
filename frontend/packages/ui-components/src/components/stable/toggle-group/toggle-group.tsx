import type { ReactNode } from "react"
import {
  ToggleGroup as CoreToggleGroup,
  ToggleGroupItem as CoreToggleGroupItem,
} from "@workspace/ui-core/components/toggle-group"

export interface ToggleGroupOption {
  value: string
  label: ReactNode
  disabled?: boolean
}

export interface ToggleGroupProps {
  value: string
  onValueChange: (value: string) => void
  options: ToggleGroupOption[]
}

export function ToggleGroup({
  value,
  onValueChange,
  options,
}: ToggleGroupProps) {
  return (
    <CoreToggleGroup type="single" value={value} onValueChange={onValueChange}>
      {options.map((option) => (
        <CoreToggleGroupItem
          key={option.value}
          value={option.value}
          disabled={option.disabled}
        >
          {option.label}
        </CoreToggleGroupItem>
      ))}
    </CoreToggleGroup>
  )
}
