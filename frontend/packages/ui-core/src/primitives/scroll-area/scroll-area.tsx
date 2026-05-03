import { ScrollArea as ScrollAreaPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { scrollAreaClassNames } from "./scroll-area.styles"
import type {
  ScrollAreaClassResolver,
  ScrollAreaProps,
  ScrollBarProps,
} from "./scroll-area.types"

function resolveStyledScrollAreaClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: ScrollAreaClassResolver
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

function ScrollArea({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  viewportClassName,
  viewportClassNameMode = "merge",
  viewportClassResolver,
  scrollbarClassName,
  scrollbarClassNameMode = "merge",
  scrollbarClassResolver,
  thumbClassName,
  thumbClassNameMode = "merge",
  thumbClassResolver,
  children,
  ...props
}: ScrollAreaProps) {
  if (mode === "primitive") {
    return (
      <ScrollAreaPrimitive.Root className={className} {...props}>
        <ScrollAreaPrimitive.Viewport className={viewportClassName}>
          {children}
        </ScrollAreaPrimitive.Viewport>
        <ScrollBar
          mode="primitive"
          className={scrollbarClassName}
          thumbClassName={thumbClassName}
        />
        <ScrollAreaPrimitive.Corner />
      </ScrollAreaPrimitive.Root>
    )
  }

  return (
    <ScrollAreaPrimitive.Root
      data-slot="scroll-area"
      className={resolveStyledScrollAreaClassName({
        className,
        defaultClassName: scrollAreaClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <ScrollAreaPrimitive.Viewport
        data-slot="scroll-area-viewport"
        className={resolveStyledScrollAreaClassName({
          className: viewportClassName,
          defaultClassName: scrollAreaClassNames.slot1,
          classNameMode: viewportClassNameMode,
          classResolver: viewportClassResolver,
        })}
      >
        {children}
      </ScrollAreaPrimitive.Viewport>
      <ScrollBar
        className={scrollbarClassName}
        classNameMode={scrollbarClassNameMode}
        classResolver={scrollbarClassResolver}
        thumbClassName={thumbClassName}
        thumbClassNameMode={thumbClassNameMode}
        thumbClassResolver={thumbClassResolver}
      />
      <ScrollAreaPrimitive.Corner />
    </ScrollAreaPrimitive.Root>
  )
}

function ScrollBar({
  mode = DEFAULT_MODE,
  className,
  orientation = "vertical",
  classNameMode = "merge",
  classResolver,
  thumbClassName,
  thumbClassNameMode = "merge",
  thumbClassResolver,
  ...props
}: ScrollBarProps) {
  if (mode === "primitive") {
    return (
      <ScrollAreaPrimitive.ScrollAreaScrollbar
        orientation={orientation}
        className={className}
        {...props}
      >
        <ScrollAreaPrimitive.ScrollAreaThumb className={thumbClassName} />
      </ScrollAreaPrimitive.ScrollAreaScrollbar>
    )
  }

  return (
    <ScrollAreaPrimitive.ScrollAreaScrollbar
      data-slot="scroll-area-scrollbar"
      data-orientation={orientation}
      orientation={orientation}
      className={resolveStyledScrollAreaClassName({
        className,
        defaultClassName: scrollAreaClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <ScrollAreaPrimitive.ScrollAreaThumb
        data-slot="scroll-area-thumb"
        className={resolveStyledScrollAreaClassName({
          className: thumbClassName,
          defaultClassName: scrollAreaClassNames.slot2,
          classNameMode: thumbClassNameMode,
          classResolver: thumbClassResolver,
        })}
      />
    </ScrollAreaPrimitive.ScrollAreaScrollbar>
  )
}

export { ScrollArea, ScrollBar }
