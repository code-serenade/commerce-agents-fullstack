import * as React from "react"
import { Label as LabelPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type LabelClassNameMode = "merge" | "replace"

export type LabelClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type LabelProps = React.ComponentProps<typeof LabelPrimitive.Root> & {
  mode?: BaseMode
  classNameMode?: LabelClassNameMode
  classResolver?: LabelClassResolver
}
