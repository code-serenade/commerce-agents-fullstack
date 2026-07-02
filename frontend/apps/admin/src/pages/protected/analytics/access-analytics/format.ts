export function formatInteger(value: number) {
  return Math.round(value).toLocaleString()
}

export function formatPercent(value: number) {
  return `${Math.round(value * 100)}%`
}

export function formatDuration(seconds: number) {
  if (seconds <= 0) return "0s"
  const minutes = Math.floor(seconds / 60)
  const restSeconds = seconds % 60
  if (minutes <= 0) return `${restSeconds}s`
  const hours = Math.floor(minutes / 60)
  const restMinutes = minutes % 60
  if (hours <= 0) return `${minutes}m ${restSeconds}s`
  return `${hours}h ${restMinutes}m`
}

export function formatDateTime(value: string | null) {
  if (!value) return "-"
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return "-"
  return date.toLocaleString()
}

export function formatAnalyticsDateLabel(value: unknown) {
  if (typeof value === "string") {
    return value.length > 5 ? value.slice(5) : value
  }

  if (Array.isArray(value)) {
    const [year, dayOrMonth, dayOfMonth] = value
    if (isFiniteNumber(year) && isFiniteNumber(dayOrMonth)) {
      if (isFiniteNumber(dayOfMonth)) {
        return formatMonthDay(dayOrMonth, dayOfMonth)
      }

      const date = new Date(Date.UTC(year, 0, 1))
      date.setUTCDate(dayOrMonth)
      return formatMonthDay(date.getUTCMonth() + 1, date.getUTCDate())
    }
  }

  return value == null ? "-" : String(value)
}

export function toStartIso(dateText: string) {
  if (!dateText) return undefined
  const date = new Date(`${dateText}T00:00:00`)
  return Number.isNaN(date.getTime()) ? undefined : date.toISOString()
}

export function toEndIso(dateText: string) {
  if (!dateText) return undefined
  const date = new Date(`${dateText}T23:59:59.999`)
  return Number.isNaN(date.getTime()) ? undefined : date.toISOString()
}

export function defaultDateRange() {
  const to = new Date()
  const from = new Date(to)
  from.setDate(to.getDate() - 29)
  return {
    from: toInputDate(from),
    to: toInputDate(to),
  }
}

export function presetDateRange(days: number) {
  const to = new Date()
  const from = new Date(to)
  from.setDate(to.getDate() - Math.max(0, days - 1))
  return {
    from: toInputDate(from),
    to: toInputDate(to),
  }
}

export function toInputDate(date: Date) {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, "0")
  const day = String(date.getDate()).padStart(2, "0")
  return `${year}-${month}-${day}`
}

function formatMonthDay(month: number, day: number) {
  return `${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value)
}
