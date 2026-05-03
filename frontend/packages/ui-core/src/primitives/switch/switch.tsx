import { Switch as SwitchPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { switchClassNames } from "./switch.styles"
import type {
  SwitchClassResolver,
  SwitchProps,
  SwitchSize,
  SwitchThumbClassResolver,
} from "./switch.types"

function resolveStyledSwitchClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
  size,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: SwitchClassResolver
  size: SwitchSize
}) {
  if (classResolver) {
    return classResolver({
      size,
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function resolveStyledSwitchThumbClassName({
  className,
  classNameMode,
  classResolver,
}: {
  className?: string
  classNameMode: "merge" | "replace"
  classResolver?: SwitchThumbClassResolver
}) {
  const defaultClassName = switchClassNames.slot1

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

function Switch({
  mode = DEFAULT_MODE,
  className,
  size = "default",
  classNameMode = "merge",
  classResolver,
  thumbClassName,
  thumbClassNameMode = "merge",
  thumbClassResolver,
  ...props
}: SwitchProps) {
  if (mode === "primitive") {
    return (
      <SwitchPrimitive.Root className={className} {...props}>
        <SwitchPrimitive.Thumb className={thumbClassName} />
      </SwitchPrimitive.Root>
    )
  }

  const resolvedSize = (size ?? "default") as SwitchSize

  return (
    <SwitchPrimitive.Root
      data-slot="switch"
      data-size={resolvedSize}
      className={resolveStyledSwitchClassName({
        className,
        defaultClassName: switchClassNames.slot2,
        classNameMode,
        classResolver,
        size: resolvedSize,
      })}
      {...props}
    >
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={resolveStyledSwitchThumbClassName({
          className: thumbClassName,
          classNameMode: thumbClassNameMode,
          classResolver: thumbClassResolver,
        })}
      />
    </SwitchPrimitive.Root>
  )
}

export { Switch }
