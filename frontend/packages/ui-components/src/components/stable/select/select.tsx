import type { ReactNode } from "react"
import {
  Select as CoreSelect,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@workspace/ui-core/components/select"

export type SelectOption = {
  label: ReactNode
  value: string
  disabled?: boolean
}

export interface SelectProps {
  value: string
  onValueChange: (value: string) => void

  disabled?: boolean
  list: SelectOption[]
  placeholder?: string
}

export function Select({
  value,
  onValueChange,
  disabled = false,
  list,
  placeholder,
}: SelectProps) {
  return (
    <CoreSelect value={value} onValueChange={onValueChange} disabled={disabled}>
      <SelectTrigger>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>

      <SelectContent>
        {list.map((item) => (
          <SelectItem
            key={item.value}
            value={item.value}
            disabled={item.disabled}
          >
            {item.label}
          </SelectItem>
        ))}
      </SelectContent>
    </CoreSelect>
  )
}
