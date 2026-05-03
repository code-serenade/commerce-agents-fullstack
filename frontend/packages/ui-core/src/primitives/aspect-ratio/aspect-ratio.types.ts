import * as React from "react"
import { AspectRatio as AspectRatioPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type AspectRatioClassNameMode = "merge" | "replace"

export type AspectRatioClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type AspectRatioProps = React.ComponentProps<
  typeof AspectRatioPrimitive.Root
> & {
  mode?: BaseMode
}
