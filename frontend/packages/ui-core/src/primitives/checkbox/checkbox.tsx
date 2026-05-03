import { Checkbox as CheckboxPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { CheckIcon } from "../../lib/icon-slots"
import { checkboxClassNames } from "./checkbox.styles"
import type { CheckboxClassResolver, CheckboxProps } from "./checkbox.types"

function DefaultCheckIcon() {
  return <CheckIcon className="size-3.5" />
}

function resolveStyledCheckboxClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: CheckboxClassResolver
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

function Checkbox({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  indicatorClassName,
  indicatorClassNameMode = "merge",
  indicatorClassResolver,
  ...props
}: CheckboxProps) {
  if (mode === "primitive") {
    return (
      <CheckboxPrimitive.Root className={className} {...props}>
        <CheckboxPrimitive.Indicator className={indicatorClassName}>
          <DefaultCheckIcon />
        </CheckboxPrimitive.Indicator>
      </CheckboxPrimitive.Root>
    )
  }

  return (
    <CheckboxPrimitive.Root
      data-slot="checkbox"
      className={resolveStyledCheckboxClassName({
        className,
        defaultClassName: checkboxClassNames.slot2,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <CheckboxPrimitive.Indicator
        data-slot="checkbox-indicator"
        className={resolveStyledCheckboxClassName({
          className: indicatorClassName,
          defaultClassName: checkboxClassNames.slot1,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
      >
        <DefaultCheckIcon />
      </CheckboxPrimitive.Indicator>
    </CheckboxPrimitive.Root>
  )
}

export { Checkbox }
