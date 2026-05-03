import * as React from "react"
import { Switch as SwitchPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type SwitchClassNameMode = "merge" | "replace"

export type SwitchClassResolver = (params: {
  size: SwitchSize
  defaultClassName: string
  className?: string
}) => string

export type SwitchThumbClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SwitchSize = "sm" | "default"

export type SwitchProps = React.ComponentProps<typeof SwitchPrimitive.Root> & {
  mode?: BaseMode
  size?: SwitchSize
  classNameMode?: SwitchClassNameMode
  classResolver?: SwitchClassResolver
  thumbClassName?: string
  thumbClassNameMode?: SwitchClassNameMode
  thumbClassResolver?: SwitchThumbClassResolver
}
