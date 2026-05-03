import * as React from "react"
import { Checkbox as CheckboxPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type CheckboxClassNameMode = "merge" | "replace"

export type CheckboxClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type CheckboxProps = React.ComponentProps<
  typeof CheckboxPrimitive.Root
> & {
  mode?: BaseMode
  classNameMode?: CheckboxClassNameMode
  classResolver?: CheckboxClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: CheckboxClassNameMode
  indicatorClassResolver?: CheckboxClassResolver
}
