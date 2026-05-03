import { enUS, zhCN as zhCNLocale } from "date-fns/locale"
import { Calendar as CoreCalendar } from "@workspace/ui-core/components/calendar"

export type CalendarMode = "single" | "multiple" | "range"

export interface CalendarDateRange {
  from?: Date
  to?: Date
}

export interface CalendarDisabled {
  before?: Date
  after?: Date
  dates?: Date[]
  daysOfWeek?: Array<0 | 1 | 2 | 3 | 4 | 5 | 6>
}

export type CalendarValue = Date | Date[] | CalendarDateRange | undefined
export type CalendarLocale = "en" | "zhCN"
export type CalendarCaptionMode = "label" | "dropdown"

export interface CalendarProps {
  mode?: CalendarMode
  value?: CalendarValue
  onValueChange?: (value: CalendarValue | undefined) => void
  required?: boolean
  defaultMonth?: Date
  month?: Date
  onMonthChange?: (month: Date) => void
  numberOfMonths?: number
  showOutsideDays?: boolean
  locale?: CalendarLocale
  captionMode?: CalendarCaptionMode
  weekStartsOn?: 0 | 1 | 2 | 3 | 4 | 5 | 6
  startMonth?: Date
  endMonth?: Date
  disabled?: boolean | CalendarDisabled
}

function resolveDisabled(
  disabled: CalendarProps["disabled"]
): unknown {
  if (disabled === undefined || typeof disabled === "boolean") {
    return disabled
  }

  const matchers: unknown[] = []

  if (disabled.before) {
    matchers.push({ before: disabled.before })
  }

  if (disabled.after) {
    matchers.push({ after: disabled.after })
  }

  if (disabled.dates?.length) {
    matchers.push(...disabled.dates)
  }

  if (disabled.daysOfWeek?.length) {
    matchers.push({ dayOfWeek: disabled.daysOfWeek })
  }

  return matchers.length > 0 ? matchers : undefined
}

function resolveLocale(locale: CalendarLocale | undefined) {
  if (locale === "zhCN") {
    return zhCNLocale
  }

  return enUS
}

function resolveSelected(
  mode: CalendarMode,
  value: CalendarValue
): Date | Date[] | CalendarDateRange | undefined {
  if (mode === "multiple") {
    return Array.isArray(value) ? value : undefined
  }

  if (mode === "range") {
    if (value && !Array.isArray(value) && !(value instanceof Date)) {
      return value
    }

    return undefined
  }

  return value instanceof Date ? value : undefined
}

export function Calendar({
  mode = "single",
  value,
  onValueChange,
  required = false,
  defaultMonth,
  month,
  onMonthChange,
  numberOfMonths = 1,
  showOutsideDays = true,
  locale = "en",
  captionMode = "label",
  weekStartsOn,
  startMonth,
  endMonth,
  disabled,
}: CalendarProps) {
  return (
    <CoreCalendar
      mode={mode}
      required={required}
      selected={resolveSelected(mode, value) as never}
      onSelect={onValueChange as never}
      defaultMonth={defaultMonth}
      month={month}
      onMonthChange={onMonthChange}
      numberOfMonths={numberOfMonths}
      showOutsideDays={showOutsideDays}
      captionLayout={captionMode}
      locale={resolveLocale(locale)}
      weekStartsOn={weekStartsOn}
      startMonth={startMonth}
      endMonth={endMonth}
      disabled={resolveDisabled(disabled) as never}
    />
  )
}
