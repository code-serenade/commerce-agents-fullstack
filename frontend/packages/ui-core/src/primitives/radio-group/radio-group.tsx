import { RadioGroup as RadioGroupPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { radioGroupClassNames } from "./radio-group.styles"
import type {
  RadioGroupClassResolver,
  RadioGroupItemProps,
  RadioGroupProps,
} from "./radio-group.types"

function resolveStyledRadioGroupClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: RadioGroupClassResolver
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

function RadioGroup({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: RadioGroupProps) {
  if (mode === "primitive") {
    return <RadioGroupPrimitive.Root className={className} {...props} />
  }

  return (
    <RadioGroupPrimitive.Root
      data-slot="radio-group"
      className={resolveStyledRadioGroupClassName({
        className,
        defaultClassName: radioGroupClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function RadioGroupItem({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  indicatorClassName,
  indicatorClassNameMode = "merge",
  indicatorClassResolver,
  dotClassName,
  dotClassNameMode = "merge",
  dotClassResolver,
  ...props
}: RadioGroupItemProps) {
  if (mode === "primitive") {
    return (
      <RadioGroupPrimitive.Item className={className} {...props}>
        <RadioGroupPrimitive.Indicator className={indicatorClassName}>
          <span className={dotClassName} />
        </RadioGroupPrimitive.Indicator>
      </RadioGroupPrimitive.Item>
    )
  }

  return (
    <RadioGroupPrimitive.Item
      data-slot="radio-group-item"
      className={resolveStyledRadioGroupClassName({
        className,
        defaultClassName: radioGroupClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <RadioGroupPrimitive.Indicator
        data-slot="radio-group-indicator"
        className={resolveStyledRadioGroupClassName({
          className: indicatorClassName,
          defaultClassName: radioGroupClassNames.slot1,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
      >
        <span
          className={resolveStyledRadioGroupClassName({
            className: dotClassName,
            defaultClassName: radioGroupClassNames.slot2,
            classNameMode: dotClassNameMode,
            classResolver: dotClassResolver,
          })}
        />
      </RadioGroupPrimitive.Indicator>
    </RadioGroupPrimitive.Item>
  )
}

export { RadioGroup, RadioGroupItem }
