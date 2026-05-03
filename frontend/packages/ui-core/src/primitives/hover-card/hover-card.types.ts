import * as React from "react"
import { HoverCard as HoverCardPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type HoverCardClassNameMode = "merge" | "replace"

export type HoverCardClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type HoverCardProps = React.ComponentProps<
  typeof HoverCardPrimitive.Root
> & {
  mode?: BaseMode
}

export type HoverCardTriggerProps = React.ComponentProps<
  typeof HoverCardPrimitive.Trigger
> & {
  mode?: BaseMode
}

export type HoverCardContentProps = React.ComponentProps<
  typeof HoverCardPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: HoverCardClassNameMode
  classResolver?: HoverCardClassResolver
}
