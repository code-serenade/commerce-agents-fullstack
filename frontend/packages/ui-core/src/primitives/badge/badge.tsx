import { Slot } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { badgeVariants } from "./badge.styles"
import type {
  BadgeClassResolver,
  BadgeProps,
  BadgeVariant,
} from "./badge.types"

function resolveStyledBadgeClassName({
  className,
  variant,
  classNameMode,
  classResolver,
}: {
  className?: string
  variant: BadgeVariant
  classNameMode: "merge" | "replace"
  classResolver?: BadgeClassResolver
}) {
  const defaultClassName = badgeVariants({ variant })

  if (classResolver) {
    return classResolver({
      variant,
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function Badge({ mode = DEFAULT_MODE, asChild = false, ...props }: BadgeProps) {
  const Comp = asChild ? Slot.Root : "span"

  if (mode === "primitive") {
    const {
      className,
      variant: _variant,
      classNameMode: _classNameMode,
      classResolver: _classResolver,
      ...rest
    } = props
    return <Comp className={className} {...rest} />
  }

  const {
    className,
    variant = "default",
    classNameMode = "merge",
    classResolver,
    ...styledProps
  } = props
  const resolvedVariant = (variant ?? "default") as BadgeVariant

  return (
    <Comp
      data-slot="badge"
      data-variant={resolvedVariant}
      className={resolveStyledBadgeClassName({
        className,
        variant: resolvedVariant,
        classNameMode,
        classResolver,
      })}
      {...styledProps}
    />
  )
}

export { Badge }
