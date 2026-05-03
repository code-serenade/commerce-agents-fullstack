import * as React from "react"

import type { BaseMode } from "../../lib/component-mode"

export type SpinnerClassNameMode = "merge" | "replace"

export type SpinnerClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SpinnerProps = React.ComponentProps<"svg"> & {
  mode?: BaseMode
  classNameMode?: SpinnerClassNameMode
  classResolver?: SpinnerClassResolver
}
