import { HoverCard as HoverCardPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { hoverCardClassNames } from "./hover-card.styles"
import type {
  HoverCardClassResolver,
  HoverCardContentProps,
  HoverCardProps,
  HoverCardTriggerProps,
} from "./hover-card.types"

function resolveStyledHoverCardClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: HoverCardClassResolver
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

function HoverCard({ mode = DEFAULT_MODE, ...props }: HoverCardProps) {
  if (mode === "primitive") {
    return <HoverCardPrimitive.Root {...props} />
  }

  return <HoverCardPrimitive.Root data-slot="hover-card" {...props} />
}

function HoverCardTrigger({
  mode = DEFAULT_MODE,
  ...props
}: HoverCardTriggerProps) {
  if (mode === "primitive") {
    return <HoverCardPrimitive.Trigger {...props} />
  }

  return (
    <HoverCardPrimitive.Trigger data-slot="hover-card-trigger" {...props} />
  )
}

function HoverCardContent({
  mode = DEFAULT_MODE,
  className,
  align = "center",
  sideOffset = 4,
  classNameMode = "merge",
  classResolver,
  ...props
}: HoverCardContentProps) {
  if (mode === "primitive") {
    return (
      <HoverCardPrimitive.Portal>
        <HoverCardPrimitive.Content
          align={align}
          sideOffset={sideOffset}
          className={className}
          {...props}
        />
      </HoverCardPrimitive.Portal>
    )
  }

  return (
    <HoverCardPrimitive.Portal data-slot="hover-card-portal">
      <HoverCardPrimitive.Content
        data-slot="hover-card-content"
        align={align}
        sideOffset={sideOffset}
        className={resolveStyledHoverCardClassName({
          className,
          defaultClassName: hoverCardClassNames.slot1,
          classNameMode,
          classResolver,
        })}
        {...props}
      />
    </HoverCardPrimitive.Portal>
  )
}

export { HoverCard, HoverCardTrigger, HoverCardContent }
