import * as React from "react"
import { Collapsible as CollapsiblePrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type CollapsibleClassNameMode = "merge" | "replace"

export type CollapsibleClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type CollapsibleProps = React.ComponentProps<
  typeof CollapsiblePrimitive.Root
> & {
  mode?: BaseMode
}

export type CollapsibleTriggerProps = React.ComponentProps<
  typeof CollapsiblePrimitive.CollapsibleTrigger
> & {
  mode?: BaseMode
}

export type CollapsibleContentProps = React.ComponentProps<
  typeof CollapsiblePrimitive.CollapsibleContent
> & {
  mode?: BaseMode
}
