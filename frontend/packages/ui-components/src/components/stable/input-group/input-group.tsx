import type { ReactNode } from "react"
import {
  InputGroup as CoreInputGroup,
  InputGroupAddon as CoreInputGroupAddon,
  InputGroupButton as CoreInputGroupButton,
  InputGroupInput as CoreInputGroupInput,
  InputGroupText as CoreInputGroupText,
} from "@workspace/ui-core/components/input-group"

export interface InputGroupProps {
  value: string
  onValueChange: (value: string) => void
  placeholder?: string
  prefix?: ReactNode
  actionLabel?: ReactNode
  onAction?: () => void
}

export function InputGroup({
  value,
  onValueChange,
  placeholder = "",
  prefix,
  actionLabel,
  onAction,
}: InputGroupProps) {
  return (
    <CoreInputGroup>
      {prefix ? (
        <CoreInputGroupAddon>
          <CoreInputGroupText>{prefix}</CoreInputGroupText>
        </CoreInputGroupAddon>
      ) : null}

      <CoreInputGroupInput
        value={value}
        placeholder={placeholder}
        onChange={(event) => onValueChange(event.target.value)}
      />

      {actionLabel ? (
        <CoreInputGroupAddon align="inline-end">
          <CoreInputGroupButton size="xs" onClick={onAction}>
            {actionLabel}
          </CoreInputGroupButton>
        </CoreInputGroupAddon>
      ) : null}
    </CoreInputGroup>
  )
}
