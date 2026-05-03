import { Switch as CoreSwitch } from "@workspace/ui-core/components/switch"

export interface SwitchProps {
  checked: boolean
  onCheckedChange?: (checked: boolean) => void
  disabled?: boolean
  size?: "sm" | "default"
}

export function Switch({
  checked,
  onCheckedChange,
  disabled = false,
  size = "default",
}: SwitchProps) {
  return (
    <CoreSwitch
      checked={checked}
      disabled={disabled}
      size={size}
      onCheckedChange={onCheckedChange}
    />
  )
}
