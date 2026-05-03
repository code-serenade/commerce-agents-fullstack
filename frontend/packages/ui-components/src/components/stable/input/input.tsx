import type { ChangeEvent } from "react"
import { Input as CoreInput } from "@workspace/ui-core/components/input"

export type InputType =
  | "text"
  | "password"
  | "email"
  | "number"
  | "search"
  | "tel"
  | "url"

export interface InputProps {
  value: string
  onValueChange: (value: string) => void
  placeholder?: string
  disabled?: boolean
  type?: InputType
}

export function Input({
  value,
  onValueChange,
  placeholder,
  disabled = false,
  type = "text",
}: InputProps) {
  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    onValueChange(event.target.value)
  }

  return (
    <CoreInput
      value={value}
      onChange={handleChange}
      placeholder={placeholder}
      disabled={disabled}
      type={type}
    />
  )
}
