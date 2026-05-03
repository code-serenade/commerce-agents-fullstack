import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { kbdClassNames } from "./kbd.styles"
import type { KbdClassResolver, KbdGroupProps, KbdProps } from "./kbd.types"

function resolveStyledKbdClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: KbdClassResolver
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

function Kbd({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: KbdProps) {
  if (mode === "primitive") {
    return <kbd className={className} {...props} />
  }

  return (
    <kbd
      data-slot="kbd"
      className={resolveStyledKbdClassName({
        className,
        defaultClassName: kbdClassNames.slot1,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function KbdGroup({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: KbdGroupProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="kbd-group"
      className={resolveStyledKbdClassName({
        className,
        defaultClassName: kbdClassNames.slot2,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { Kbd, KbdGroup }
