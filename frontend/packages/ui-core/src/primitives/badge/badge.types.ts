import * as React from "react"
import type { VariantProps } from "class-variance-authority"

import type { BaseMode } from "../../lib/component-mode"
import { badgeVariants } from "./badge.styles"

export type BadgeVariantProps = VariantProps<typeof badgeVariants>
export type BadgeVariant = NonNullable<BadgeVariantProps["variant"]>

export type BadgeClassNameMode = "merge" | "replace"

export type BadgeClassResolver = (params: {
  variant: BadgeVariant
  defaultClassName: string
  className?: string
}) => string

export type BadgeProps = React.ComponentProps<"span"> &
  BadgeVariantProps & {
    asChild?: boolean
    mode?: BaseMode
    classNameMode?: BadgeClassNameMode
    classResolver?: BadgeClassResolver
  }
