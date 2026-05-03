import type { CSSProperties, ReactNode } from "react"
import {
  ChartContainer as CoreChartContainer,
  type ChartConfig,
} from "@workspace/ui-core/primitives/chart"

export type { ChartConfig }

export interface ChartSurfaceProps {
  config: ChartConfig
  children: ReactNode
  height?: number
}

export function ChartSurface({
  config,
  children,
  height = 260,
}: ChartSurfaceProps) {
  return (
    <CoreChartContainer
      config={config}
      className="h-[var(--chart-height)] w-full"
      style={{ "--chart-height": `${height}px` } as CSSProperties}
    >
      {children}
    </CoreChartContainer>
  )
}
