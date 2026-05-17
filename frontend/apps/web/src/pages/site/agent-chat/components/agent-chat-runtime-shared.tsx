import type { ReactNode } from "react"
import { Check } from "lucide-react"
import { NativeSelect } from "@workspace/ui-components/stable/native-select"

export function OptionSegment({
  disabled,
  icon,
  label,
  onValueChange,
  options,
  value,
}: {
  disabled: boolean
  icon?: ReactNode
  label: string
  onValueChange: (value: string) => void
  options: { description?: string; label: string; value: string }[]
  value: string
}) {
  return (
    <div className="grid gap-2">
      <div className="flex items-center gap-1.5 text-xs font-semibold text-foreground">
        {icon}
        <span>{label}</span>
      </div>
      <div className="grid gap-2 sm:grid-cols-3">
        {options.map((option) => {
          const active = option.value === value
          return (
            <button
              key={option.value}
              type="button"
              disabled={disabled}
              onClick={() => onValueChange(option.value)}
              className={`min-w-0 rounded-lg border px-3 py-2.5 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
                active
                  ? "border-primary/50 bg-primary/10 text-primary"
                  : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
              }`}
              title={option.description}
            >
              <span className="flex items-center gap-2 text-sm font-semibold">
                {active ? <Check className="size-3.5 shrink-0" /> : null}
                <span className="truncate">{option.label}</span>
              </span>
              {option.description ? (
                <span className="mt-1 block text-xs/5 opacity-80">
                  {option.description}
                </span>
              ) : null}
            </button>
          )
        })}
      </div>
    </div>
  )
}

export function RuntimeSelect({
  disabled,
  label,
  onValueChange,
  options,
  value,
}: {
  disabled: boolean
  label: string
  onValueChange: (value: string) => void
  options: { label: string; value: string }[]
  value: string
}) {
  return (
    <label className="flex min-w-0 items-center gap-1.5 text-xs text-muted-foreground">
      <span className="shrink-0">{label}</span>
      <NativeSelect
        disabled={disabled || options.length === 0}
        onValueChange={onValueChange}
        options={options}
        value={value}
      />
    </label>
  )
}
