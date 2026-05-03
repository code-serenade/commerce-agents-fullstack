import {
  NativeSelect as CoreNativeSelect,
  NativeSelectOption as CoreNativeSelectOption,
} from "@workspace/ui-core/components/native-select"

export interface NativeSelectOption {
  label: string
  value: string
  disabled?: boolean
}

export interface NativeSelectProps {
  value: string
  onValueChange: (value: string) => void
  options: NativeSelectOption[]
  placeholder?: string
  disabled?: boolean
}

export function NativeSelect({
  value,
  onValueChange,
  options,
  placeholder,
  disabled = false,
}: NativeSelectProps) {
  return (
    <CoreNativeSelect
      value={value}
      disabled={disabled}
      onChange={(event) => onValueChange(event.target.value)}
    >
      {placeholder ? (
        <CoreNativeSelectOption value="" disabled>
          {placeholder}
        </CoreNativeSelectOption>
      ) : null}

      {options.map((option) => (
        <CoreNativeSelectOption
          key={option.value}
          value={option.value}
          disabled={option.disabled}
        >
          {option.label}
        </CoreNativeSelectOption>
      ))}
    </CoreNativeSelect>
  )
}
