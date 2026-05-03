import type { ReactNode } from "react"
import {
  Select as CoreSelect,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@workspace/ui-core/components/select"

export interface DataTableInlineSelectOption {
  label: ReactNode
  value: string
  disabled?: boolean
}

export interface DataTableInlineSelectProps {
  value: string
  onValueChange: (value: string) => void
  disabled?: boolean
  placeholder?: string
  options: readonly DataTableInlineSelectOption[]
}

export function DataTableInlineSelect({
  value,
  onValueChange,
  disabled = false,
  placeholder,
  options,
}: DataTableInlineSelectProps) {
  return (
    <CoreSelect value={value} onValueChange={onValueChange} disabled={disabled}>
      <SelectTrigger
        hideIndicator
        className="h-8 w-24 border-0 bg-transparent px-0 text-sm text-muted-foreground shadow-none hover:bg-transparent focus:ring-0 focus-visible:ring-0"
      >
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>

      <SelectContent>
        {options.map((option) => (
          <SelectItem
            key={option.value}
            value={option.value}
            disabled={option.disabled}
          >
            {option.label}
          </SelectItem>
        ))}
      </SelectContent>
    </CoreSelect>
  )
}
