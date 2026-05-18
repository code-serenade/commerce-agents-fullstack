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
    <div className="grid gap-1.5 sm:grid-cols-[4.75rem_minmax(0,1fr)] sm:items-center">
      <div className="flex min-w-0 items-center gap-1.5 text-xs font-semibold text-foreground">
        {icon}
        <span className="truncate">{label}</span>
      </div>
      <div className="grid gap-1.5 sm:grid-cols-3">
        {options.map((option) => {
          const active = option.value === value
          return (
            <button
              key={option.value}
              type="button"
              disabled={disabled}
              onClick={() => onValueChange(option.value)}
              className={`flex h-9 min-w-0 items-center gap-1.5 rounded-md border px-2 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
                active
                  ? "border-primary/50 bg-primary/10 text-primary"
                  : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
              }`}
              title={option.description}
              aria-pressed={active}
            >
              <span className="flex size-3.5 shrink-0 items-center justify-center">
                {active ? <Check className="size-3.5" /> : null}
              </span>
              <span className="min-w-0 truncate text-sm font-semibold">
                {option.label}
              </span>
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
