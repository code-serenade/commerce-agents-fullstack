import * as React from "react"
import { Progress as ProgressPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type ProgressClassNameMode = "merge" | "replace"

export type ProgressClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type ProgressProps = React.ComponentProps<
  typeof ProgressPrimitive.Root
> & {
  mode?: BaseMode
  classNameMode?: ProgressClassNameMode
  classResolver?: ProgressClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: ProgressClassNameMode
  indicatorClassResolver?: ProgressClassResolver
}
