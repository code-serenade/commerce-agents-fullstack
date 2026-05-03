import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { Loader2Icon } from "../../lib/icon-slots"
import { spinnerClassNames } from "./spinner.styles"
import type { SpinnerClassResolver, SpinnerProps } from "./spinner.types"

function resolveStyledSpinnerClassName({
  className,
  classNameMode,
  classResolver,
}: {
  className?: string
  classNameMode: "merge" | "replace"
  classResolver?: SpinnerClassResolver
}) {
  const defaultClassName = spinnerClassNames.slot1

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

function Spinner({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: SpinnerProps) {
  if (mode === "primitive") {
    return (
      <Loader2Icon
        role="status"
        aria-label="Loading"
        className={className}
        {...props}
      />
    )
  }

  return (
    <Loader2Icon
      role="status"
      aria-label="Loading"
      className={resolveStyledSpinnerClassName({
        className,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { Spinner }
