import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts"
import {
  ChartLegend as CoreChartLegend,
  ChartLegendContent as CoreChartLegendContent,
  ChartTooltip as CoreChartTooltip,
  ChartTooltipContent as CoreChartTooltipContent,
} from "@workspace/ui-core/primitives/chart"
import { createChartConfig, type ChartSeriesConfig } from "./chart-config"
import { ChartSurface } from "./chart-surface"

export type TrendChartSeries = ChartSeriesConfig

export interface TrendLineChartProps {
  data: Array<Record<string, number | string>>
  xKey: string
  series: TrendChartSeries[]
  height?: number
}

export function TrendLineChart({
  data,
  xKey,
  series,
  height = 260,
}: TrendLineChartProps) {
  const config = createChartConfig(series)

  return (
    <ChartSurface config={config} height={height}>
      <LineChart data={data} margin={{ left: 4, right: 16, top: 12 }}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey={xKey}
          tickLine={false}
          axisLine={false}
          tickMargin={10}
        />
        <YAxis tickLine={false} axisLine={false} tickMargin={10} width={36} />
        <CoreChartTooltip content={<CoreChartTooltipContent />} />
        <CoreChartLegend content={<CoreChartLegendContent />} />
        {series.map((item) => (
          <Line
            key={item.key}
            dataKey={item.key}
            type="monotone"
            stroke={`var(--color-${item.key})`}
            strokeWidth={2}
            dot={false}
          />
        ))}
      </LineChart>
    </ChartSurface>
  )
}
