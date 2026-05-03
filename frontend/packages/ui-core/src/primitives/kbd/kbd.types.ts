import * as React from "react"

import type { BaseMode } from "../../lib/component-mode"

export type KbdClassNameMode = "merge" | "replace"

export type KbdClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type KbdProps = React.ComponentProps<"kbd"> & {
  mode?: BaseMode
  classNameMode?: KbdClassNameMode
  classResolver?: KbdClassResolver
}

export type KbdGroupProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: KbdClassNameMode
  classResolver?: KbdClassResolver
}
