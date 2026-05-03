"use client"

import { Progress as ProgressPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { progressClassNames } from "./progress.styles"
import type { ProgressClassResolver, ProgressProps } from "./progress.types"

function resolveStyledProgressClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: ProgressClassResolver
}) {
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

function Progress({
  mode = DEFAULT_MODE,
  className,
  value,
  classNameMode = "merge",
  classResolver,
  indicatorClassName,
  indicatorClassNameMode = "merge",
  indicatorClassResolver,
  ...props
}: ProgressProps) {
  if (mode === "primitive") {
    return (
      <ProgressPrimitive.Root className={className} {...props}>
        <ProgressPrimitive.Indicator
          className={indicatorClassName}
          style={{ transform: `translateX(-${100 - (value || 0)}%)` }}
        />
      </ProgressPrimitive.Root>
    )
  }

  return (
    <ProgressPrimitive.Root
      data-slot="progress"
      className={resolveStyledProgressClassName({
        className,
        defaultClassName: progressClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <ProgressPrimitive.Indicator
        data-slot="progress-indicator"
        className={resolveStyledProgressClassName({
          className: indicatorClassName,
          defaultClassName: progressClassNames.slot1,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
        style={{ transform: `translateX(-${100 - (value || 0)}%)` }}
      />
    </ProgressPrimitive.Root>
  )
}

export { Progress }
