import type { ReactNode } from "react"
import { NativeSelect } from "@workspace/ui-components/stable/native-select"

export type RuntimeControlSize = "default" | "compact"

export function SettingsRow({
  align = "center",
  children,
  label,
  size = "default",
}: {
  align?: "center" | "start"
  children: ReactNode
  label: string
  size?: RuntimeControlSize
}) {
  const rowClassName =
    align === "start"
      ? "grid gap-1.5 sm:grid-cols-[7.5rem_minmax(0,1fr)] sm:gap-2 sm:items-start"
      : "grid gap-1.5 sm:grid-cols-[7.5rem_minmax(0,1fr)] sm:gap-2 sm:items-center"
  const labelClassName =
    size === "compact"
      ? "min-w-0 text-xs/5 font-medium text-foreground"
      : "min-w-0 text-sm/6 font-semibold text-foreground"

  return (
    <div className={rowClassName}>
      <div className={labelClassName}>
        <span className="wrap-break-word">{label}</span>
      </div>
      <div className="min-w-0">{children}</div>
    </div>
  )
}

export function OptionSegment({
  disabled,
  columns = 3,
  label,
  onValueChange,
  options,
  size = "default",
  value,
}: {
  disabled: boolean
  columns?: 2 | 3
  label: string
  onValueChange: (value: string) => void
  options: { description?: string; label: string; value: string }[]
  size?: RuntimeControlSize
  value: string
}) {
  const optionTextClassName =
    size === "compact"
      ? "min-w-0 text-xs/tight font-medium"
      : "min-w-0 text-xs/tight font-semibold sm:text-sm/tight"

  return (
    <SettingsRow align="start" label={label} size={size}>
      <div
        className={
          columns === 2
            ? "grid grid-cols-2 gap-1.5"
            : "grid grid-cols-1 gap-1.5 sm:grid-cols-3"
        }
      >
        {options.map((option) => {
          const active = option.value === value
          return (
            <button
              key={option.value}
              type="button"
              disabled={disabled}
              onClick={() => onValueChange(option.value)}
              className={`flex min-h-8 min-w-0 items-center justify-center rounded-lg border px-2 py-1 text-center transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
                active
                  ? "border-primary/60 bg-primary/10 text-primary ring-1 ring-primary/10"
                  : "border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-muted-foreground hover:bg-(--agent-chat-accent-soft) hover:text-foreground"
              }`}
              title={option.description}
              aria-pressed={active}
            >
              <span className={optionTextClassName}>{option.label}</span>
            </button>
          )
        })}
      </div>
    </SettingsRow>
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
      {label ? <span className="shrink-0">{label}</span> : null}
      <NativeSelect
        disabled={disabled || options.length === 0}
        onValueChange={onValueChange}
        options={options}
        value={value}
      />
    </label>
  )
}
