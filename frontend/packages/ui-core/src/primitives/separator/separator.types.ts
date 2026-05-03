import * as React from "react"
import { Separator as SeparatorPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type SeparatorClassNameMode = "merge" | "replace"

export type SeparatorClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SeparatorProps = React.ComponentProps<
  typeof SeparatorPrimitive.Root
> & {
  mode?: BaseMode
  classNameMode?: SeparatorClassNameMode
  classResolver?: SeparatorClassResolver
}
