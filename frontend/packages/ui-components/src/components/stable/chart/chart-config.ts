import type { ChartConfig } from "./chart-surface"

export interface ChartSeriesConfig {
  key: string
  label: string
  color?: string
}

export function createChartConfig(items: ChartSeriesConfig[]) {
  return items.reduce<ChartConfig>((acc, item, index) => {
    acc[item.key] = {
      label: item.label,
      color: item.color ?? `var(--chart-${(index % 5) + 1})`,
    }
    return acc
  }, {})
}
