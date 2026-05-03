import { Checkbox as CoreCheckbox } from "@workspace/ui-core/components/checkbox"

export interface CheckboxProps {
  checked: boolean | "indeterminate"
  onCheckedChange?: (checked: boolean | "indeterminate") => void
  disabled?: boolean
}

export function Checkbox({
  checked,
  onCheckedChange,
  disabled = false,
}: CheckboxProps) {
  return (
    <CoreCheckbox
      checked={checked}
      disabled={disabled}
      onCheckedChange={onCheckedChange}
    />
  )
}
