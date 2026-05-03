import { Cell, Pie, PieChart } from "recharts"
import {
  ChartLegend as CoreChartLegend,
  ChartLegendContent as CoreChartLegendContent,
  ChartTooltip as CoreChartTooltip,
  ChartTooltipContent as CoreChartTooltipContent,
} from "@workspace/ui-core/primitives/chart"
import { createChartConfig } from "./chart-config"
import { ChartSurface } from "./chart-surface"

export interface DonutChartProps {
  data: Array<{
    key: string
    label: string
    value: number
    color?: string
  }>
  height?: number
}

export function DonutChart({ data, height = 260 }: DonutChartProps) {
  const config = createChartConfig(data)

  return (
    <ChartSurface config={config} height={height}>
      <PieChart>
        <CoreChartTooltip content={<CoreChartTooltipContent hideLabel />} />
        <Pie
          data={data}
          dataKey="value"
          nameKey="key"
          innerRadius={64}
          outerRadius={92}
          paddingAngle={3}
        >
          {data.map((item) => (
            <Cell key={item.key} fill={`var(--color-${item.key})`} />
          ))}
        </Pie>
        <CoreChartLegend
          content={<CoreChartLegendContent nameKey="key" />}
          verticalAlign="bottom"
        />
      </PieChart>
    </ChartSurface>
  )
}
