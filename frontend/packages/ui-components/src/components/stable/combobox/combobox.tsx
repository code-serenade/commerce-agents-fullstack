import type { ReactNode } from "react"
import {
  Combobox as CoreCombobox,
  ComboboxContent as CoreComboboxContent,
  ComboboxEmpty as CoreComboboxEmpty,
  ComboboxInput as CoreComboboxInput,
  ComboboxItem as CoreComboboxItem,
  ComboboxList as CoreComboboxList,
} from "@workspace/ui-core/components/combobox"

export interface ComboboxOption {
  label: ReactNode
  value: string
  disabled?: boolean
}

export interface ComboboxProps {
  value: string | null
  onValueChange: (value: string | null) => void
  options: ComboboxOption[]
  placeholder?: string
  disabled?: boolean
  emptyLabel?: ReactNode
  allowClear?: boolean
}

export function Combobox({
  value,
  onValueChange,
  options,
  placeholder = "Search...",
  disabled = false,
  emptyLabel = "No result.",
  allowClear = true,
}: ComboboxProps) {
  return (
    <CoreCombobox value={value} onValueChange={onValueChange} items={options}>
      <CoreComboboxInput
        placeholder={placeholder}
        disabled={disabled}
        showClear={allowClear}
      />
      <CoreComboboxContent>
        <CoreComboboxEmpty>{emptyLabel}</CoreComboboxEmpty>
        <CoreComboboxList>
          {options.map((item) => (
            <CoreComboboxItem
              key={item.value}
              value={item.value}
              disabled={item.disabled}
            >
              {item.label}
            </CoreComboboxItem>
          ))}
        </CoreComboboxList>
      </CoreComboboxContent>
    </CoreCombobox>
  )
}
