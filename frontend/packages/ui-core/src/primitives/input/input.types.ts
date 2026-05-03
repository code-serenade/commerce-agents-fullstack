import * as React from "react"
import type { BaseMode } from "../../lib/component-mode"

export type InputClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type InputProps = React.ComponentProps<"input"> & {
  mode?: BaseMode
  classNameMode?: "merge" | "replace"
  classResolver?: InputClassResolver
}
