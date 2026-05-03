import { Slot } from "radix-ui"
import { Separator as SeparatorPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { Separator } from "../separator"
import {
  buttonGroupBaseClassName,
  buttonGroupOrientationClassNames,
  buttonGroupSeparatorClassName,
  buttonGroupTextClassName,
} from "./button-group.styles"
import type {
  ButtonGroupClassResolver,
  ButtonGroupOrientation,
  ButtonGroupProps,
  ButtonGroupSeparatorProps,
  ButtonGroupTextProps,
} from "./button-group.types"

function resolveStyledClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: (params: {
    defaultClassName: string
    className?: string
  }) => string
}) {
  if (classResolver) {
    return classResolver({ defaultClassName, className })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function resolveStyledButtonGroupClassName({
  className,
  orientation,
  classNameMode,
  classResolver,
}: {
  className?: string
  orientation: ButtonGroupOrientation
  classNameMode: "merge" | "replace"
  classResolver?: ButtonGroupClassResolver
}) {
  const defaultClassName = cn(
    buttonGroupBaseClassName,
    buttonGroupOrientationClassNames[orientation]
  )

  if (classResolver) {
    return classResolver({ orientation, defaultClassName, className })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function ButtonGroup({
  className,
  orientation = "horizontal",
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: ButtonGroupProps) {
  const resolvedOrientation = orientation as ButtonGroupOrientation

  if (mode === "primitive") {
    const rest = { ...props }
    return <div role="group" className={className} {...rest} />
  }

  return (
    <div
      role="group"
      data-slot="button-group"
      data-orientation={resolvedOrientation}
      className={resolveStyledButtonGroupClassName({
        className,
        orientation: resolvedOrientation,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function ButtonGroupText({
  className,
  asChild = false,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: ButtonGroupTextProps) {
  const Comp = asChild ? Slot.Root : "div"

  if (mode === "primitive") {
    const rest = { ...props }
    return <Comp className={className} {...rest} />
  }

  return (
    <Comp
      className={resolveStyledClassName({
        className,
        defaultClassName: buttonGroupTextClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function ButtonGroupSeparator({
  className,
  mode = DEFAULT_MODE,
  orientation = "vertical",
  classNameMode = "merge",
  classResolver,
  ...props
}: ButtonGroupSeparatorProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return (
      <SeparatorPrimitive.Root
        orientation={orientation}
        className={className}
        {...rest}
      />
    )
  }

  return (
    <Separator
      data-slot="button-group-separator"
      orientation={orientation}
      className={resolveStyledClassName({
        className,
        defaultClassName: buttonGroupSeparatorClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { ButtonGroup, ButtonGroupSeparator, ButtonGroupText }
