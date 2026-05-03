"use client"

import * as ResizablePrimitive from "react-resizable-panels"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { resizableClassNames } from "./resizable.styles"
import type {
  ResizableClassResolver,
  ResizableHandleProps,
  ResizablePanelGroupProps,
  ResizablePanelProps,
} from "./resizable.types"

function resolveStyledResizableClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: ResizableClassResolver
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

function ResizablePanelGroup({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: ResizablePanelGroupProps) {
  if (mode === "primitive") {
    return <ResizablePrimitive.Group className={className} {...props} />
  }

  return (
    <ResizablePrimitive.Group
      data-slot="resizable-panel-group"
      className={resolveStyledResizableClassName({
        className,
        defaultClassName: resizableClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function ResizablePanel({
  mode = DEFAULT_MODE,
  ...props
}: ResizablePanelProps) {
  if (mode === "primitive") {
    return <ResizablePrimitive.Panel {...props} />
  }

  return <ResizablePrimitive.Panel data-slot="resizable-panel" {...props} />
}

function ResizableHandle({
  mode = DEFAULT_MODE,
  withHandle,
  className,
  classNameMode = "merge",
  classResolver,
  handleClassName,
  handleClassNameMode = "merge",
  handleClassResolver,
  ...props
}: ResizableHandleProps) {
  if (mode === "primitive") {
    return (
      <ResizablePrimitive.Separator className={className} {...props}>
        {withHandle && <div className={handleClassName} />}
      </ResizablePrimitive.Separator>
    )
  }

  return (
    <ResizablePrimitive.Separator
      data-slot="resizable-handle"
      className={resolveStyledResizableClassName({
        className,
        defaultClassName: resizableClassNames.slot2,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {withHandle && (
        <div
          className={resolveStyledResizableClassName({
            className: handleClassName,
            defaultClassName: resizableClassNames.slot1,
            classNameMode: handleClassNameMode,
            classResolver: handleClassResolver,
          })}
        />
      )}
    </ResizablePrimitive.Separator>
  )
}

export { ResizableHandle, ResizablePanel, ResizablePanelGroup }
