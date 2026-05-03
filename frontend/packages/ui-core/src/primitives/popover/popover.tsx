import { Popover as PopoverPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { popoverClassNames } from "./popover.styles"
import type {
  PopoverAnchorProps,
  PopoverClassResolver,
  PopoverContentProps,
  PopoverDescriptionProps,
  PopoverHeaderProps,
  PopoverProps,
  PopoverTitleProps,
  PopoverTriggerProps,
} from "./popover.types"

function resolveStyledPopoverClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: PopoverClassResolver
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

function Popover({ mode = DEFAULT_MODE, ...props }: PopoverProps) {
  if (mode === "primitive") {
    return <PopoverPrimitive.Root {...props} />
  }

  return <PopoverPrimitive.Root data-slot="popover" {...props} />
}

function PopoverTrigger({
  mode = DEFAULT_MODE,
  ...props
}: PopoverTriggerProps) {
  if (mode === "primitive") {
    return <PopoverPrimitive.Trigger {...props} />
  }

  return <PopoverPrimitive.Trigger data-slot="popover-trigger" {...props} />
}

function PopoverContent({
  mode = DEFAULT_MODE,
  className,
  align = "center",
  sideOffset = 4,
  classNameMode = "merge",
  classResolver,
  ...props
}: PopoverContentProps) {
  if (mode === "primitive") {
    return (
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          align={align}
          sideOffset={sideOffset}
          className={className}
          {...props}
        />
      </PopoverPrimitive.Portal>
    )
  }

  return (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Content
        data-slot="popover-content"
        align={align}
        sideOffset={sideOffset}
        className={resolveStyledPopoverClassName({
          className,
          defaultClassName: popoverClassNames.slot1,
          classNameMode,
          classResolver,
        })}
        {...props}
      />
    </PopoverPrimitive.Portal>
  )
}

function PopoverAnchor({ mode = DEFAULT_MODE, ...props }: PopoverAnchorProps) {
  if (mode === "primitive") {
    return <PopoverPrimitive.Anchor {...props} />
  }

  return <PopoverPrimitive.Anchor data-slot="popover-anchor" {...props} />
}

function PopoverHeader({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: PopoverHeaderProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="popover-header"
      className={resolveStyledPopoverClassName({
        className,
        defaultClassName: popoverClassNames.slot2,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function PopoverTitle({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: PopoverTitleProps) {
  if (mode === "primitive") {
    return <h2 className={className} {...props} />
  }

  return (
    <h2
      data-slot="popover-title"
      className={resolveStyledPopoverClassName({
        className,
        defaultClassName: popoverClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function PopoverDescription({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: PopoverDescriptionProps) {
  if (mode === "primitive") {
    return <p className={className} {...props} />
  }

  return (
    <p
      data-slot="popover-description"
      className={resolveStyledPopoverClassName({
        className,
        defaultClassName: popoverClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export {
  Popover,
  PopoverAnchor,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
}
