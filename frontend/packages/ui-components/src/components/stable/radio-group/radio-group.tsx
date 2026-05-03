import type { ReactNode } from "react"
import {
  RadioGroup as CoreRadioGroup,
  RadioGroupItem as CoreRadioGroupItem,
} from "@workspace/ui-core/components/radio-group"

export interface RadioGroupProps {
  value: string
  onValueChange: (value: string) => void
  disabled?: boolean
  children: ReactNode
}

export interface RadioGroupItemProps {
  value: string
  disabled?: boolean
  children?: ReactNode
}

export function RadioGroup({
  value,
  onValueChange,
  disabled = false,
  children,
}: RadioGroupProps) {
  return (
    <CoreRadioGroup value={value} onValueChange={onValueChange} disabled={disabled}>
      {children}
    </CoreRadioGroup>
  )
}

export function RadioGroupItem({
  value,
  disabled = false,
  children,
}: RadioGroupItemProps) {
  return (
    <label className="inline-flex items-center gap-2">
      <CoreRadioGroupItem value={value} disabled={disabled} />
      {children}
    </label>
  )
}
