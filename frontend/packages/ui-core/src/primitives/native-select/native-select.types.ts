import * as React from "react"

import type { BaseMode } from "../../lib/component-mode"

export type NativeSelectClassNameMode = "merge" | "replace"

export type NativeSelectClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type NativeSelectSize = "sm" | "default"

export type NativeSelectProps = Omit<React.ComponentProps<"select">, "size"> & {
  mode?: BaseMode
  size?: NativeSelectSize
  className?: string
  classNameMode?: NativeSelectClassNameMode
  classResolver?: NativeSelectClassResolver
  selectClassName?: string
  selectClassNameMode?: NativeSelectClassNameMode
  selectClassResolver?: NativeSelectClassResolver
  iconClassName?: string
  iconClassNameMode?: NativeSelectClassNameMode
  iconClassResolver?: NativeSelectClassResolver
}

export type NativeSelectOptionProps = React.ComponentProps<"option"> & {
  mode?: BaseMode
}

export type NativeSelectOptGroupProps = React.ComponentProps<"optgroup"> & {
  mode?: BaseMode
  classNameMode?: NativeSelectClassNameMode
  classResolver?: NativeSelectClassResolver
}
