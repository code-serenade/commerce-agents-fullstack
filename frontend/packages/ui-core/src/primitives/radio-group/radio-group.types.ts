import * as React from "react"
import { RadioGroup as RadioGroupPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type RadioGroupClassNameMode = "merge" | "replace"

export type RadioGroupClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type RadioGroupProps = React.ComponentProps<
  typeof RadioGroupPrimitive.Root
> & {
  mode?: BaseMode
  classNameMode?: RadioGroupClassNameMode
  classResolver?: RadioGroupClassResolver
}

export type RadioGroupItemProps = React.ComponentProps<
  typeof RadioGroupPrimitive.Item
> & {
  mode?: BaseMode
  classNameMode?: RadioGroupClassNameMode
  classResolver?: RadioGroupClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: RadioGroupClassNameMode
  indicatorClassResolver?: RadioGroupClassResolver
  dotClassName?: string
  dotClassNameMode?: RadioGroupClassNameMode
  dotClassResolver?: RadioGroupClassResolver
}
