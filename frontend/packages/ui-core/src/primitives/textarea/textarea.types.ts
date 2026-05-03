import * as React from "react"

import type { BaseMode } from "../../lib/component-mode"

export type TextareaClassNameMode = "merge" | "replace"

export type TextareaClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type TextareaProps = React.ComponentProps<"textarea"> & {
  mode?: BaseMode
  classNameMode?: TextareaClassNameMode
  classResolver?: TextareaClassResolver
}
