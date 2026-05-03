"use client"

import { Toggle as TogglePrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { toggleVariants } from "./toggle.styles"
import type {
  ToggleClassResolver,
  ToggleProps,
  ToggleSize,
  ToggleVariant,
} from "./toggle.types"

function resolveStyledToggleClassName({
  className,
  variant,
  size,
  classNameMode,
  classResolver,
}: {
  className?: string
  variant: ToggleVariant
  size: ToggleSize
  classNameMode: "merge" | "replace"
  classResolver?: ToggleClassResolver
}) {
  const defaultClassName = toggleVariants({ variant, size })

  if (classResolver) {
    return classResolver({
      variant,
      size,
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function Toggle({ mode = DEFAULT_MODE, ...props }: ToggleProps) {
  if (mode === "primitive") {
    const {
      className,
      variant: _variant,
      size: _size,
      classNameMode: _classNameMode,
      classResolver: _classResolver,
      ...rest
    } = props
    return <TogglePrimitive.Root className={className} {...rest} />
  }

  const {
    className,
    variant = "default",
    size = "default",
    classNameMode = "merge",
    classResolver,
    ...styledProps
  } = props
  const resolvedVariant = (variant ?? "default") as ToggleVariant
  const resolvedSize = (size ?? "default") as ToggleSize

  return (
    <TogglePrimitive.Root
      data-slot="toggle"
      data-variant={resolvedVariant}
      data-size={resolvedSize}
      className={resolveStyledToggleClassName({
        className,
        variant: resolvedVariant,
        size: resolvedSize,
        classNameMode,
        classResolver,
      })}
      {...styledProps}
    />
  )
}

export { Toggle }
