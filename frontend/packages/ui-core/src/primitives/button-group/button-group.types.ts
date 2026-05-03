import * as React from "react"
import type { BaseMode } from "../../lib/component-mode"
import { Separator } from "../separator"

export type ButtonGroupOrientation = "horizontal" | "vertical"

export type ButtonGroupClassResolver = (params: {
  orientation: ButtonGroupOrientation
  defaultClassName: string
  className?: string
}) => string

export type ButtonGroupSlotClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type ButtonGroupProps = React.ComponentProps<"div"> & {
  orientation?: ButtonGroupOrientation
  mode?: BaseMode
  classNameMode?: "merge" | "replace"
  classResolver?: ButtonGroupClassResolver
}

export type ButtonGroupTextProps = React.ComponentProps<"div"> & {
  asChild?: boolean
  mode?: BaseMode
  classNameMode?: "merge" | "replace"
  classResolver?: ButtonGroupSlotClassResolver
}

export type ButtonGroupSeparatorProps = React.ComponentProps<
  typeof Separator
> & {
  mode?: BaseMode
  classNameMode?: "merge" | "replace"
  classResolver?: ButtonGroupSlotClassResolver
}
