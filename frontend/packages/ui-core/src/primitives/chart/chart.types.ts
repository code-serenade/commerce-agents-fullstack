import * as React from "react"
import * as RechartsPrimitive from "recharts"
import type { TooltipValueType } from "recharts"

import type { BaseMode } from "../../lib/component-mode"

export type ChartClassNameMode = "merge" | "replace"

export type ChartClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

type TooltipNameType = number | string

export type ChartConfig = Record<
  string,
  {
    label?: React.ReactNode
    icon?: React.ComponentType
  } & (
    | { color?: string; theme?: never }
    | { color?: never; theme: Record<"light" | "dark", string> }
  )
>

export type ChartContainerProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  config: ChartConfig
  children: React.ComponentProps<
    typeof RechartsPrimitive.ResponsiveContainer
  >["children"]
  initialDimension?: {
    width: number
    height: number
  }
  classNameMode?: ChartClassNameMode
  classResolver?: ChartClassResolver
}

export type ChartStyleProps = {
  id: string
  config: ChartConfig
}

export type ChartTooltipContentProps = React.ComponentProps<
  typeof RechartsPrimitive.Tooltip
> &
  React.ComponentProps<"div"> & {
    mode?: BaseMode
    hideLabel?: boolean
    hideIndicator?: boolean
    indicator?: "line" | "dot" | "dashed"
    nameKey?: string
    labelKey?: string
    classNameMode?: ChartClassNameMode
    classResolver?: ChartClassResolver
    labelClassNameMode?: ChartClassNameMode
    labelClassResolver?: ChartClassResolver
    valueClassName?: string
    valueClassNameMode?: ChartClassNameMode
    valueClassResolver?: ChartClassResolver
  } & Omit<
    RechartsPrimitive.DefaultTooltipContentProps<
      TooltipValueType,
      TooltipNameType
    >,
    "accessibilityLayer"
  >

export type ChartLegendContentProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  hideIcon?: boolean
  nameKey?: string
  classNameMode?: ChartClassNameMode
  classResolver?: ChartClassResolver
  itemClassName?: string
  itemClassNameMode?: ChartClassNameMode
  itemClassResolver?: ChartClassResolver
} & RechartsPrimitive.DefaultLegendContentProps
