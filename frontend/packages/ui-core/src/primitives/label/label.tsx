import { Label as LabelPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { labelClassNames } from "./label.styles"
import type { LabelClassResolver, LabelProps } from "./label.types"

function resolveStyledLabelClassName({
  className,
  classNameMode,
  classResolver,
}: {
  className?: string
  classNameMode: "merge" | "replace"
  classResolver?: LabelClassResolver
}) {
  const defaultClassName = labelClassNames.slot1

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

function Label({ mode = DEFAULT_MODE, ...props }: LabelProps) {
  if (mode === "primitive") {
    const {
      className,
      classNameMode: _classNameMode,
      classResolver: _classResolver,
      ...rest
    } = props
    return <LabelPrimitive.Root className={className} {...rest} />
  }

  const {
    className,
    classNameMode = "merge",
    classResolver,
    ...styledProps
  } = props

  return (
    <LabelPrimitive.Root
      data-slot="label"
      className={resolveStyledLabelClassName({
        className,
        classNameMode,
        classResolver,
      })}
      {...styledProps}
    />
  )
}

export { Label }
