import {
  Time as CoreTime,
  type TimeValue as CoreTimeValue,
} from "@workspace/ui-core/components/time"

export interface TimeValue {
  hour: string
  minute: string
  second?: string
}

export type TimeSize = "sm" | "md" | "lg"

export interface TimeProps {
  value?: TimeValue
  defaultValue?: TimeValue
  onValueChange?: (value: TimeValue) => void
  disabled?: boolean
  showSeconds?: boolean
  ariaLabel?: string
  size?: TimeSize
}

function getSizeClasses(size: TimeSize) {
  if (size === "sm") {
    return {
      itemHeight: 19,
      columnClassName:
        "relative h-[95px] w-[58px] overflow-y-auto border-r border-[color-mix(in_oklab,var(--foreground)_8%,transparent)] bg-transparent px-0 py-0 snap-y snap-mandatory overscroll-contain [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden focus:outline-none last:border-r-0",
      itemClassName:
        "pointer-events-none relative z-10 flex h-[19px] snap-center items-center justify-center text-[11px] font-semibold tracking-0 text-[var(--muted-foreground)] transition-all duration-150 aria-selected:scale-100 aria-selected:text-[var(--accent-foreground)] aria-selected:[text-shadow:0_1px_1px_rgba(0,0,0,0.08)] [&:not([aria-selected='true'])]:scale-[0.98]",
      selectionClassName:
        "inset-x-0 h-[17px] rounded-[5px] bg-[var(--surface-active)] shadow-[inset_0_1px_0_color-mix(in_oklab,white_12%,transparent)]",
    }
  }

  if (size === "lg") {
    return {
      itemHeight: 27,
      columnClassName:
        "relative h-[135px] w-[72px] overflow-y-auto border-r border-[color-mix(in_oklab,var(--foreground)_8%,transparent)] bg-transparent px-0 py-0 snap-y snap-mandatory overscroll-contain [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden focus:outline-none last:border-r-0",
      itemClassName:
        "pointer-events-none relative z-10 flex h-[27px] snap-center items-center justify-center text-[13px] font-semibold tracking-0 text-[var(--muted-foreground)] transition-all duration-150 aria-selected:scale-100 aria-selected:text-[var(--accent-foreground)] aria-selected:[text-shadow:0_1px_1px_rgba(0,0,0,0.08)] [&:not([aria-selected='true'])]:scale-[0.98]",
      selectionClassName:
        "inset-x-0 h-[25px] rounded-[7px] bg-[var(--surface-active)] shadow-[inset_0_1px_0_color-mix(in_oklab,white_12%,transparent)]",
    }
  }

  return {
    itemHeight: 23,
    columnClassName:
      "relative h-[115px] w-[64px] overflow-y-auto border-r border-[color-mix(in_oklab,var(--foreground)_8%,transparent)] bg-transparent px-0 py-0 snap-y snap-mandatory overscroll-contain [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden focus:outline-none last:border-r-0",
    itemClassName:
      "pointer-events-none relative z-10 flex h-[23px] snap-center items-center justify-center text-[12px] font-semibold tracking-0 text-[var(--muted-foreground)] transition-all duration-150 aria-selected:scale-100 aria-selected:text-[var(--accent-foreground)] aria-selected:[text-shadow:0_1px_1px_rgba(0,0,0,0.08)] [&:not([aria-selected='true'])]:scale-[0.98]",
    selectionClassName:
      "inset-x-0 h-[21px] rounded-[6px] bg-[var(--surface-active)] shadow-[inset_0_1px_0_color-mix(in_oklab,white_12%,transparent)]",
  }
}

function toCoreValue(value: TimeValue | undefined): CoreTimeValue | undefined {
  if (!value) {
    return undefined
  }

  return {
    hour: value.hour,
    minute: value.minute,
    second: value.second,
  }
}

function fromCoreValue(value: CoreTimeValue): TimeValue {
  return {
    hour: value.hour,
    minute: value.minute,
    second: value.second,
  }
}

export function Time({
  value,
  defaultValue,
  onValueChange,
  disabled = false,
  showSeconds = true,
  ariaLabel = "Time",
  size = "md",
}: TimeProps) {
  const sizeClasses = getSizeClasses(size)

  return (
    <CoreTime
      mode="primitive"
      value={toCoreValue(value)}
      defaultValue={toCoreValue(defaultValue)}
      onValueChange={(nextValue) => {
        onValueChange?.(fromCoreValue(nextValue))
      }}
      disabled={disabled}
      showSeconds={showSeconds}
      visibleRows={5}
      itemHeight={sizeClasses.itemHeight}
      hourLabel="Hours"
      minuteLabel="Minutes"
      secondLabel="Seconds"
      aria-label={ariaLabel}
      className="flex items-center justify-center gap-0 overflow-hidden rounded-[min(var(--radius-xl),1.25rem)] bg-[var(--surface)]"
      columnClassName={sizeClasses.columnClassName}
      itemClassName={sizeClasses.itemClassName}
      selectionClassName={sizeClasses.selectionClassName}
    />
  )
}
