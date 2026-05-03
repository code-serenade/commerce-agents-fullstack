import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { textareaClassNames } from "./textarea.styles"
import type { TextareaClassResolver, TextareaProps } from "./textarea.types"

function resolveStyledTextareaClassName({
  className,
  classNameMode,
  classResolver,
}: {
  className?: string
  classNameMode: "merge" | "replace"
  classResolver?: TextareaClassResolver
}) {
  const defaultClassName = textareaClassNames.slot1

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

function Textarea({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TextareaProps) {
  if (mode === "primitive") {
    return <textarea className={className} {...props} />
  }

  return (
    <textarea
      data-slot="textarea"
      className={resolveStyledTextareaClassName({
        className,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { Textarea }
