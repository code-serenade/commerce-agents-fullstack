import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { inputClassName } from "./input.styles"
import type { InputClassResolver, InputProps } from "./input.types"

function resolveStyledInputClassName({
  className,
  classNameMode,
  classResolver,
}: {
  className?: string
  classNameMode: "merge" | "replace"
  classResolver?: InputClassResolver
}) {
  if (classResolver) {
    return classResolver({
      defaultClassName: inputClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? inputClassName
  }

  return cn(inputClassName, className)
}

export function Input({
  mode = DEFAULT_MODE,
  className,
  type,
  classNameMode = "merge",
  classResolver,
  ...props
}: InputProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return <input type={type} className={className} {...rest} />
  }

  const resolvedClassName = resolveStyledInputClassName({
    className,
    classNameMode,
    classResolver,
  })

  return (
    <input
      type={type}
      data-slot="input"
      className={resolvedClassName}
      {...props}
    />
  )
}
