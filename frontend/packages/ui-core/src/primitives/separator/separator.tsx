"use client"

import { Separator as SeparatorPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { separatorClassNames } from "./separator.styles"
import type { SeparatorClassResolver, SeparatorProps } from "./separator.types"

function resolveStyledSeparatorClassName({
  className,
  classNameMode,
  classResolver,
}: {
  className?: string
  classNameMode: "merge" | "replace"
  classResolver?: SeparatorClassResolver
}) {
  const defaultClassName = separatorClassNames.slot1

  if (classResolver) {
    return classResolver({
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function Separator({
  mode = DEFAULT_MODE,
  className,
  orientation = "horizontal",
  decorative = true,
  classNameMode = "merge",
  classResolver,
  ...props
}: SeparatorProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return (
      <SeparatorPrimitive.Root
        decorative={decorative}
        orientation={orientation}
        className={className}
        {...rest}
      />
    )
  }

  return (
    <SeparatorPrimitive.Root
      data-slot="separator"
      decorative={decorative}
      orientation={orientation}
      className={resolveStyledSeparatorClassName({
        className,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { Separator }
