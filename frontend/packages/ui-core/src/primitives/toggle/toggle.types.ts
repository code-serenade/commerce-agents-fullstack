import * as React from "react"
import type { VariantProps } from "class-variance-authority"
import { Toggle as TogglePrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"
import { toggleVariants } from "./toggle.styles"

export type ToggleVariantProps = VariantProps<typeof toggleVariants>
export type ToggleVariant = NonNullable<ToggleVariantProps["variant"]>
export type ToggleSize = NonNullable<ToggleVariantProps["size"]>

export type ToggleClassNameMode = "merge" | "replace"

export type ToggleClassResolver = (params: {
  variant: ToggleVariant
  size: ToggleSize
  defaultClassName: string
  className?: string
}) => string

export type ToggleProps = React.ComponentProps<typeof TogglePrimitive.Root> &
  ToggleVariantProps & {
    mode?: BaseMode
    classNameMode?: ToggleClassNameMode
    classResolver?: ToggleClassResolver
  }
