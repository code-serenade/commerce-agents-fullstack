import { Drawer as DrawerPrimitive } from "vaul"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { drawerClassNames } from "./drawer.styles"
import type {
  DrawerClassResolver,
  DrawerCloseProps,
  DrawerContentProps,
  DrawerDescriptionProps,
  DrawerFooterProps,
  DrawerHeaderProps,
  DrawerOverlayProps,
  DrawerPortalProps,
  DrawerProps,
  DrawerTitleProps,
  DrawerTriggerProps,
} from "./drawer.types"

function resolveStyledDrawerClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: DrawerClassResolver
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

function Drawer({ mode = DEFAULT_MODE, ...props }: DrawerProps) {
  if (mode === "primitive") {
    return <DrawerPrimitive.Root {...props} />
  }

  return <DrawerPrimitive.Root data-slot="drawer" {...props} />
}

function DrawerTrigger({ mode = DEFAULT_MODE, ...props }: DrawerTriggerProps) {
  if (mode === "primitive") {
    return <DrawerPrimitive.Trigger {...props} />
  }

  return <DrawerPrimitive.Trigger data-slot="drawer-trigger" {...props} />
}

function DrawerPortal({ mode = DEFAULT_MODE, ...props }: DrawerPortalProps) {
  if (mode === "primitive") {
    return <DrawerPrimitive.Portal {...props} />
  }

  return <DrawerPrimitive.Portal data-slot="drawer-portal" {...props} />
}

function DrawerClose({ mode = DEFAULT_MODE, ...props }: DrawerCloseProps) {
  if (mode === "primitive") {
    return <DrawerPrimitive.Close {...props} />
  }

  return <DrawerPrimitive.Close data-slot="drawer-close" {...props} />
}

function DrawerOverlay({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DrawerOverlayProps) {
  if (mode === "primitive") {
    return <DrawerPrimitive.Overlay className={className} {...props} />
  }

  return (
    <DrawerPrimitive.Overlay
      data-slot="drawer-overlay"
      className={resolveStyledDrawerClassName({
        className,
        defaultClassName: drawerClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DrawerContent({
  mode = DEFAULT_MODE,
  className,
  children,
  showHandle = true,
  classNameMode = "merge",
  classResolver,
  overlayClassName,
  overlayClassNameMode = "merge",
  overlayClassResolver,
  handleClassName,
  handleClassNameMode = "merge",
  handleClassResolver,
  ...props
}: DrawerContentProps) {
  if (mode === "primitive") {
    return (
      <DrawerPrimitive.Portal>
        <DrawerPrimitive.Overlay className={overlayClassName} />
        <DrawerPrimitive.Content className={className} {...props}>
          {showHandle && <div className={handleClassName} />}
          {children}
        </DrawerPrimitive.Content>
      </DrawerPrimitive.Portal>
    )
  }

  const resolvedOverlayClassName = resolveStyledDrawerClassName({
    className: overlayClassName,
    defaultClassName: drawerClassNames.slot0,
    classNameMode: overlayClassNameMode,
    classResolver: overlayClassResolver,
  })
  const resolvedContentClassName = resolveStyledDrawerClassName({
    className,
    defaultClassName: drawerClassNames.slot1,
    classNameMode,
    classResolver,
  })
  const resolvedHandleClassName = resolveStyledDrawerClassName({
    className: handleClassName,
    defaultClassName: drawerClassNames.slot2,
    classNameMode: handleClassNameMode,
    classResolver: handleClassResolver,
  })

  return (
    <DrawerPrimitive.Portal data-slot="drawer-portal">
      <DrawerPrimitive.Overlay
        data-slot="drawer-overlay"
        className={resolvedOverlayClassName}
      />
      <DrawerPrimitive.Content
        data-slot="drawer-content"
        className={resolvedContentClassName}
        {...props}
      >
        {showHandle && <div className={resolvedHandleClassName} />}
        {children}
      </DrawerPrimitive.Content>
    </DrawerPrimitive.Portal>
  )
}

function DrawerHeader({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DrawerHeaderProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="drawer-header"
      className={resolveStyledDrawerClassName({
        className,
        defaultClassName: drawerClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DrawerFooter({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DrawerFooterProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="drawer-footer"
      className={resolveStyledDrawerClassName({
        className,
        defaultClassName: drawerClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DrawerTitle({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DrawerTitleProps) {
  if (mode === "primitive") {
    return <DrawerPrimitive.Title className={className} {...props} />
  }

  return (
    <DrawerPrimitive.Title
      data-slot="drawer-title"
      className={resolveStyledDrawerClassName({
        className,
        defaultClassName: drawerClassNames.slot5,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DrawerDescription({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DrawerDescriptionProps) {
  if (mode === "primitive") {
    return <DrawerPrimitive.Description className={className} {...props} />
  }

  return (
    <DrawerPrimitive.Description
      data-slot="drawer-description"
      className={resolveStyledDrawerClassName({
        className,
        defaultClassName: drawerClassNames.slot6,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export {
  Drawer,
  DrawerPortal,
  DrawerOverlay,
  DrawerTrigger,
  DrawerClose,
  DrawerContent,
  DrawerHeader,
  DrawerFooter,
  DrawerTitle,
  DrawerDescription,
}
