import { Tooltip as TooltipPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { tooltipClassNames } from "./tooltip.styles"
import type {
  TooltipClassResolver,
  TooltipContentProps,
  TooltipProps,
  TooltipProviderProps,
  TooltipTriggerProps,
} from "./tooltip.types"

function resolveStyledTooltipClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: TooltipClassResolver
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

function TooltipProvider({
  mode = DEFAULT_MODE,
  delayDuration = 0,
  ...props
}: TooltipProviderProps) {
  if (mode === "primitive") {
    return (
      <TooltipPrimitive.Provider delayDuration={delayDuration} {...props} />
    )
  }

  return (
    <TooltipPrimitive.Provider
      data-slot="tooltip-provider"
      delayDuration={delayDuration}
      {...props}
    />
  )
}

function Tooltip({ mode = DEFAULT_MODE, ...props }: TooltipProps) {
  if (mode === "primitive") {
    return <TooltipPrimitive.Root {...props} />
  }

  return <TooltipPrimitive.Root data-slot="tooltip" {...props} />
}

function TooltipTrigger({
  mode = DEFAULT_MODE,
  ...props
}: TooltipTriggerProps) {
  if (mode === "primitive") {
    return <TooltipPrimitive.Trigger {...props} />
  }

  return <TooltipPrimitive.Trigger data-slot="tooltip-trigger" {...props} />
}

function TooltipContent({
  mode = DEFAULT_MODE,
  className,
  sideOffset = 0,
  classNameMode = "merge",
  classResolver,
  arrowClassName,
  arrowClassNameMode = "merge",
  arrowClassResolver,
  children,
  ...props
}: TooltipContentProps) {
  if (mode === "primitive") {
    return (
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content
          sideOffset={sideOffset}
          className={className}
          {...props}
        >
          {children}
          <TooltipPrimitive.Arrow className={arrowClassName} />
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    )
  }

  return (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Content
        data-slot="tooltip-content"
        sideOffset={sideOffset}
        className={resolveStyledTooltipClassName({
          className,
          defaultClassName: tooltipClassNames.slot2,
          classNameMode,
          classResolver,
        })}
        {...props}
      >
        {children}
        <TooltipPrimitive.Arrow
          className={resolveStyledTooltipClassName({
            className: arrowClassName,
            defaultClassName: tooltipClassNames.slot1,
            classNameMode: arrowClassNameMode,
            classResolver: arrowClassResolver,
          })}
        />
      </TooltipPrimitive.Content>
    </TooltipPrimitive.Portal>
  )
}

export { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger }
