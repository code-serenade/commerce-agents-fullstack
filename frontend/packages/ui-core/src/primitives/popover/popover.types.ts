import * as React from "react"
import { Popover as PopoverPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type PopoverClassNameMode = "merge" | "replace"

export type PopoverClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type PopoverProps = React.ComponentProps<
  typeof PopoverPrimitive.Root
> & {
  mode?: BaseMode
}

export type PopoverTriggerProps = React.ComponentProps<
  typeof PopoverPrimitive.Trigger
> & {
  mode?: BaseMode
}

export type PopoverAnchorProps = React.ComponentProps<
  typeof PopoverPrimitive.Anchor
> & {
  mode?: BaseMode
}

export type PopoverContentProps = React.ComponentProps<
  typeof PopoverPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: PopoverClassNameMode
  classResolver?: PopoverClassResolver
}

export type PopoverHeaderProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: PopoverClassNameMode
  classResolver?: PopoverClassResolver
}

export type PopoverTitleProps = React.ComponentProps<"h2"> & {
  mode?: BaseMode
  classNameMode?: PopoverClassNameMode
  classResolver?: PopoverClassResolver
}

export type PopoverDescriptionProps = React.ComponentProps<"p"> & {
  mode?: BaseMode
  classNameMode?: PopoverClassNameMode
  classResolver?: PopoverClassResolver
}
