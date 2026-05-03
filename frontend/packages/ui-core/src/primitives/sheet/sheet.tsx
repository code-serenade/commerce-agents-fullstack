"use client"

import { Dialog as SheetPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { Button } from "../button"
import { XIcon } from "../../lib/icon-slots"
import { sheetClassNames } from "./sheet.styles"
import type {
  SheetClassResolver,
  SheetCloseProps,
  SheetContentProps,
  SheetDescriptionProps,
  SheetFooterProps,
  SheetHeaderProps,
  SheetOverlayProps,
  SheetPortalProps,
  SheetProps,
  SheetSide,
  SheetTitleProps,
  SheetTriggerProps,
} from "./sheet.types"

function resolveStyledSheetClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: SheetClassResolver
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

function Sheet({ mode = DEFAULT_MODE, ...props }: SheetProps) {
  if (mode === "primitive") {
    return <SheetPrimitive.Root {...props} />
  }

  return <SheetPrimitive.Root data-slot="sheet" {...props} />
}

function SheetTrigger({ mode = DEFAULT_MODE, ...props }: SheetTriggerProps) {
  if (mode === "primitive") {
    return <SheetPrimitive.Trigger {...props} />
  }

  return <SheetPrimitive.Trigger data-slot="sheet-trigger" {...props} />
}

function SheetClose({ mode = DEFAULT_MODE, ...props }: SheetCloseProps) {
  if (mode === "primitive") {
    return <SheetPrimitive.Close {...props} />
  }

  return <SheetPrimitive.Close data-slot="sheet-close" {...props} />
}

function SheetPortal({ mode = DEFAULT_MODE, ...props }: SheetPortalProps) {
  if (mode === "primitive") {
    return <SheetPrimitive.Portal {...props} />
  }

  return <SheetPrimitive.Portal data-slot="sheet-portal" {...props} />
}

function SheetOverlay({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: SheetOverlayProps) {
  if (mode === "primitive") {
    return <SheetPrimitive.Overlay className={className} {...props} />
  }

  return (
    <SheetPrimitive.Overlay
      data-slot="sheet-overlay"
      className={resolveStyledSheetClassName({
        className,
        defaultClassName: sheetClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function SheetContent({
  mode = DEFAULT_MODE,
  className,
  children,
  side = "right",
  showCloseButton = true,
  classNameMode = "merge",
  classResolver,
  overlayClassName,
  overlayClassNameMode = "merge",
  overlayClassResolver,
  closeButtonClassName,
  closeButtonClassNameMode = "merge",
  closeButtonClassResolver,
  closeTextClassName,
  closeTextClassNameMode = "merge",
  closeTextClassResolver,
  ...props
}: SheetContentProps) {
  const resolvedSide = (side ?? "right") as SheetSide

  if (mode === "primitive") {
    return (
      <SheetPrimitive.Portal>
        <SheetPrimitive.Overlay className={overlayClassName} />
        <SheetPrimitive.Content className={className} {...props}>
          {children}
          {showCloseButton && (
            <SheetPrimitive.Close asChild>
              <Button mode="primitive" className={closeButtonClassName}>
                <XIcon />
                <span className={closeTextClassName}>Close</span>
              </Button>
            </SheetPrimitive.Close>
          )}
        </SheetPrimitive.Content>
      </SheetPrimitive.Portal>
    )
  }

  const resolvedOverlayClassName = resolveStyledSheetClassName({
    className: overlayClassName,
    defaultClassName: sheetClassNames.slot0,
    classNameMode: overlayClassNameMode,
    classResolver: overlayClassResolver,
  })
  const resolvedContentClassName = resolveStyledSheetClassName({
    className,
    defaultClassName: sheetClassNames.slot1,
    classNameMode,
    classResolver,
  })
  const resolvedCloseButtonClassName = resolveStyledSheetClassName({
    className: closeButtonClassName,
    defaultClassName: sheetClassNames.slot6,
    classNameMode: closeButtonClassNameMode,
    classResolver: closeButtonClassResolver,
  })
  const resolvedCloseTextClassName = resolveStyledSheetClassName({
    className: closeTextClassName,
    defaultClassName: sheetClassNames.slot7,
    classNameMode: closeTextClassNameMode,
    classResolver: closeTextClassResolver,
  })

  return (
    <SheetPrimitive.Portal data-slot="sheet-portal">
      <SheetPrimitive.Overlay
        data-slot="sheet-overlay"
        className={resolvedOverlayClassName}
      />
      <SheetPrimitive.Content
        data-slot="sheet-content"
        data-side={resolvedSide}
        className={resolvedContentClassName}
        {...props}
      >
        {children}
        {showCloseButton && (
          <SheetPrimitive.Close data-slot="sheet-close" asChild>
            <Button
              variant="ghost"
              className={resolvedCloseButtonClassName}
              size="icon-sm"
            >
              <XIcon />
              <span className={resolvedCloseTextClassName}>Close</span>
            </Button>
          </SheetPrimitive.Close>
        )}
      </SheetPrimitive.Content>
    </SheetPrimitive.Portal>
  )
}

function SheetHeader({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: SheetHeaderProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="sheet-header"
      className={resolveStyledSheetClassName({
        className,
        defaultClassName: sheetClassNames.slot2,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function SheetFooter({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: SheetFooterProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="sheet-footer"
      className={resolveStyledSheetClassName({
        className,
        defaultClassName: sheetClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function SheetTitle({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: SheetTitleProps) {
  if (mode === "primitive") {
    return <SheetPrimitive.Title className={className} {...props} />
  }

  return (
    <SheetPrimitive.Title
      data-slot="sheet-title"
      className={resolveStyledSheetClassName({
        className,
        defaultClassName: sheetClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function SheetDescription({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: SheetDescriptionProps) {
  if (mode === "primitive") {
    return <SheetPrimitive.Description className={className} {...props} />
  }

  return (
    <SheetPrimitive.Description
      data-slot="sheet-description"
      className={resolveStyledSheetClassName({
        className,
        defaultClassName: sheetClassNames.slot5,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export {
  Sheet,
  SheetTrigger,
  SheetClose,
  SheetPortal,
  SheetOverlay,
  SheetContent,
  SheetHeader,
  SheetFooter,
  SheetTitle,
  SheetDescription,
}
