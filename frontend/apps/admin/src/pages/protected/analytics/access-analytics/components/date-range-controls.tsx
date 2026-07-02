import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import { RefreshCw } from "lucide-react"
import {
  DateRangePicker,
  type DateRangeValue,
} from "@workspace/app-kit"
import { Button } from "@workspace/ui-components"
import { presetDateRange, toInputDate } from "../format"
import type { AnalyticsDateRange } from "../types"

interface DateRangeControlsProps {
  onRangeChange: (range: AnalyticsDateRange) => void
  onRefresh: () => Promise<void>
  range: AnalyticsDateRange
}

export function DateRangeControls({
  onRangeChange,
  onRefresh,
  range,
}: DateRangeControlsProps) {
  const { t } = useTranslation()
  const pickerValue = useMemo(() => toDateRangePickerValue(range), [range])

  return (
    <div className="flex flex-col gap-2 sm:items-end">
      <DateRangePresets range={range} onRangeChange={onRangeChange} />
      <div className="flex flex-wrap items-center gap-2">
        <DateRangePicker
          value={pickerValue}
          onValueChange={(value) => {
            onRangeChange(fromDateRangePickerValue(value, range))
          }}
          placeholder={t("admin.analytics.filters.rangePlaceholder")}
          className="min-w-[260px]"
        />
        <Button size="sm" variant="outline" onClick={() => void onRefresh()}>
          <RefreshCw className="size-4" />
          {t("admin.analytics.filters.refresh")}
        </Button>
      </div>
      <p className="text-xs text-(--app-muted-text)">
        {t("admin.analytics.filters.rangeHint")}
      </p>
    </div>
  )
}

function DateRangePresets({
  onRangeChange,
  range,
}: {
  onRangeChange: (range: AnalyticsDateRange) => void
  range: AnalyticsDateRange
}) {
  const { t } = useTranslation()
  const presets = [
    { days: 1, key: "today" },
    { days: 7, key: "last7Days" },
    { days: 30, key: "last30Days" },
  ]

  return (
    <div
      className="flex flex-wrap gap-1"
      aria-label={t("admin.analytics.filters.presets.label")}
    >
      {presets.map((preset) => {
        const nextRange = presetDateRange(preset.days)
        const isActive =
          nextRange.from === range.from && nextRange.to === range.to

        return (
          <Button
            key={preset.key}
            size="sm"
            variant={isActive ? "primary" : "outline"}
            onClick={() => onRangeChange(nextRange)}
          >
            {t(`admin.analytics.filters.presets.${preset.key}`)}
          </Button>
        )
      })}
    </div>
  )
}

function toDateRangePickerValue(range: AnalyticsDateRange): DateRangeValue {
  return {
    from: parseDate(range.from),
    to: parseDate(range.to),
  }
}

function fromDateRangePickerValue(
  value: DateRangeValue | undefined,
  fallback: AnalyticsDateRange
): AnalyticsDateRange {
  return {
    from: value?.from ? toInputDate(value.from) : fallback.from,
    to: value?.to ? toInputDate(value.to) : value?.from
      ? toInputDate(value.from)
      : fallback.to,
  }
}

function parseDate(value: string) {
  if (!value) return undefined
  const date = new Date(`${value}T00:00:00`)
  return Number.isNaN(date.getTime()) ? undefined : date
}
