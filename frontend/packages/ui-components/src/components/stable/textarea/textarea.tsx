import type { ChangeEvent } from "react"
import { Textarea as CoreTextarea } from "@workspace/ui-core/components/textarea"

export interface TextareaProps {
  value: string
  onValueChange: (value: string) => void
  placeholder?: string
  disabled?: boolean
  rows?: number
}

export function Textarea({
  value,
  onValueChange,
  placeholder,
  disabled = false,
  rows = 4,
}: TextareaProps) {
  const handleChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
    onValueChange(event.target.value)
  }

  return (
    <CoreTextarea
      value={value}
      onChange={handleChange}
      placeholder={placeholder}
      disabled={disabled}
      rows={rows}
    />
  )
}
