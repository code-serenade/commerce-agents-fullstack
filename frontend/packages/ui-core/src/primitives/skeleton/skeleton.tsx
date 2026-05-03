import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { skeletonClassNames } from "./skeleton.styles"
import type { SkeletonClassResolver, SkeletonProps } from "./skeleton.types"

function resolveStyledSkeletonClassName({
  className,
  classNameMode,
  classResolver,
}: {
  className?: string
  classNameMode: "merge" | "replace"
  classResolver?: SkeletonClassResolver
}) {
  const defaultClassName = skeletonClassNames.slot1

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

function Skeleton({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: SkeletonProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="skeleton"
      className={resolveStyledSkeletonClassName({
        className,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { Skeleton }
