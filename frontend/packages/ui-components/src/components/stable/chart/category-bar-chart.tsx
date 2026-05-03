import { Bar, BarChart, CartesianGrid, LabelList, XAxis, YAxis } from "recharts"
import {
  ChartTooltip as CoreChartTooltip,
  ChartTooltipContent as CoreChartTooltipContent,
} from "@workspace/ui-core/primitives/chart"
import type { ChartConfig } from "./chart-surface"
import { ChartSurface } from "./chart-surface"

export interface CategoryBarChartProps {
  data: Array<Record<string, number | string>>
  xKey: string
  valueKey: string
  label?: string
  color?: string
  height?: number
}

export function CategoryBarChart({
  data,
  xKey,
  valueKey,
  label = "Value",
  color = "var(--chart-1)",
  height = 260,
}: CategoryBarChartProps) {
  const config: ChartConfig = {
    [valueKey]: {
      label,
      color,
    },
  }

  return (
    <ChartSurface config={config} height={height}>
      <BarChart data={data} margin={{ left: 4, right: 16, top: 12 }}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey={xKey}
          tickLine={false}
          axisLine={false}
          tickMargin={10}
          tick={{ fill: "var(--foreground)" }}
        />
        <YAxis
          tickLine={false}
          axisLine={false}
          tickMargin={10}
          width={36}
        />
        <CoreChartTooltip content={<CoreChartTooltipContent />} />
        <Bar dataKey={valueKey} fill={`var(--color-${valueKey})`} radius={6}>
          <LabelList
            dataKey={valueKey}
            position="top"
            offset={8}
            fill="var(--foreground)"
            fontSize={12}
          />
        </Bar>
      </BarChart>
    </ChartSurface>
  )
}
