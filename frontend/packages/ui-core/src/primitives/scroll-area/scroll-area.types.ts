import * as React from "react"
import { ScrollArea as ScrollAreaPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type ScrollAreaClassNameMode = "merge" | "replace"

export type ScrollAreaClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type ScrollAreaProps = React.ComponentProps<
  typeof ScrollAreaPrimitive.Root
> & {
  mode?: BaseMode
  classNameMode?: ScrollAreaClassNameMode
  classResolver?: ScrollAreaClassResolver
  viewportClassName?: string
  viewportClassNameMode?: ScrollAreaClassNameMode
  viewportClassResolver?: ScrollAreaClassResolver
  scrollbarClassName?: string
  scrollbarClassNameMode?: ScrollAreaClassNameMode
  scrollbarClassResolver?: ScrollAreaClassResolver
  thumbClassName?: string
  thumbClassNameMode?: ScrollAreaClassNameMode
  thumbClassResolver?: ScrollAreaClassResolver
}

export type ScrollBarProps = React.ComponentProps<
  typeof ScrollAreaPrimitive.ScrollAreaScrollbar
> & {
  mode?: BaseMode
  classNameMode?: ScrollAreaClassNameMode
  classResolver?: ScrollAreaClassResolver
  thumbClassName?: string
  thumbClassNameMode?: ScrollAreaClassNameMode
  thumbClassResolver?: ScrollAreaClassResolver
}
