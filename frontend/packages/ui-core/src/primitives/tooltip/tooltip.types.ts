import * as React from "react"
import { Tooltip as TooltipPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type TooltipClassNameMode = "merge" | "replace"

export type TooltipClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type TooltipProviderProps = React.ComponentProps<
  typeof TooltipPrimitive.Provider
> & {
  mode?: BaseMode
}

export type TooltipProps = React.ComponentProps<
  typeof TooltipPrimitive.Root
> & {
  mode?: BaseMode
}

export type TooltipTriggerProps = React.ComponentProps<
  typeof TooltipPrimitive.Trigger
> & {
  mode?: BaseMode
}

export type TooltipContentProps = React.ComponentProps<
  typeof TooltipPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: TooltipClassNameMode
  classResolver?: TooltipClassResolver
  arrowClassName?: string
  arrowClassNameMode?: TooltipClassNameMode
  arrowClassResolver?: TooltipClassResolver
}
