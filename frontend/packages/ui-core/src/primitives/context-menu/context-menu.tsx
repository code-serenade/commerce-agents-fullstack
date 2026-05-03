import { ContextMenu as ContextMenuPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { ChevronRightIcon, CheckIcon } from "../../lib/icon-slots"
import { contextMenuClassNames } from "./context-menu.styles"
import type {
  ContextMenuCheckboxItemProps,
  ContextMenuClassResolver,
  ContextMenuContentProps,
  ContextMenuGroupProps,
  ContextMenuItemProps,
  ContextMenuItemVariant,
  ContextMenuLabelProps,
  ContextMenuPortalProps,
  ContextMenuProps,
  ContextMenuRadioGroupProps,
  ContextMenuRadioItemProps,
  ContextMenuSeparatorProps,
  ContextMenuShortcutProps,
  ContextMenuSubContentProps,
  ContextMenuSubProps,
  ContextMenuSubTriggerProps,
  ContextMenuTriggerProps,
} from "./context-menu.types"

function resolveStyledContextMenuClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: ContextMenuClassResolver
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

function ContextMenu({ mode = DEFAULT_MODE, ...props }: ContextMenuProps) {
  if (mode === "primitive") {
    return <ContextMenuPrimitive.Root {...props} />
  }

  return <ContextMenuPrimitive.Root data-slot="context-menu" {...props} />
}

function ContextMenuTrigger({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: ContextMenuTriggerProps) {
  const defaultClassName = "select-none"

  if (mode === "primitive") {
    return <ContextMenuPrimitive.Trigger className={className} {...props} />
  }

  return (
    <ContextMenuPrimitive.Trigger
      data-slot="context-menu-trigger"
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function ContextMenuGroup({
  mode = DEFAULT_MODE,
  ...props
}: ContextMenuGroupProps) {
  if (mode === "primitive") {
    return <ContextMenuPrimitive.Group {...props} />
  }

  return (
    <ContextMenuPrimitive.Group data-slot="context-menu-group" {...props} />
  )
}

function ContextMenuPortal({
  mode = DEFAULT_MODE,
  ...props
}: ContextMenuPortalProps) {
  if (mode === "primitive") {
    return <ContextMenuPrimitive.Portal {...props} />
  }

  return (
    <ContextMenuPrimitive.Portal data-slot="context-menu-portal" {...props} />
  )
}

function ContextMenuSub({
  mode = DEFAULT_MODE,
  ...props
}: ContextMenuSubProps) {
  if (mode === "primitive") {
    return <ContextMenuPrimitive.Sub {...props} />
  }

  return <ContextMenuPrimitive.Sub data-slot="context-menu-sub" {...props} />
}

function ContextMenuRadioGroup({
  mode = DEFAULT_MODE,
  ...props
}: ContextMenuRadioGroupProps) {
  if (mode === "primitive") {
    return <ContextMenuPrimitive.RadioGroup {...props} />
  }

  return (
    <ContextMenuPrimitive.RadioGroup
      data-slot="context-menu-radio-group"
      {...props}
    />
  )
}

function ContextMenuContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: ContextMenuContentProps) {
  const defaultClassName =
    "z-50 max-h-(--radix-context-menu-content-available-height) min-w-36 origin-(--radix-context-menu-content-transform-origin) overflow-x-hidden overflow-y-auto rounded-lg bg-popover p-1 text-popover-foreground shadow-md ring-1 ring-foreground/10 duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95"

  if (mode === "primitive") {
    return (
      <ContextMenuPrimitive.Portal>
        <ContextMenuPrimitive.Content className={className} {...props} />
      </ContextMenuPrimitive.Portal>
    )
  }

  return (
    <ContextMenuPrimitive.Portal>
      <ContextMenuPrimitive.Content
        data-slot="context-menu-content"
        className={resolveStyledContextMenuClassName({
          className,
          defaultClassName,
          classNameMode,
          classResolver,
        })}
        {...props}
      />
    </ContextMenuPrimitive.Portal>
  )
}

function ContextMenuItem({
  mode = DEFAULT_MODE,
  className,
  inset,
  variant = "default",
  classNameMode = "merge",
  classResolver,
  ...props
}: ContextMenuItemProps) {
  const defaultClassName =
    "group/context-menu-item relative flex cursor-default items-center gap-1.5 rounded-md px-1.5 py-1 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground data-inset:pl-7 data-[variant=destructive]:text-destructive data-[variant=destructive]:focus:bg-destructive/10 data-[variant=destructive]:focus:text-destructive dark:data-[variant=destructive]:focus:bg-destructive/20 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 focus:*:[svg]:text-accent-foreground data-[variant=destructive]:*:[svg]:text-destructive"
  const resolvedVariant = (variant ?? "default") as ContextMenuItemVariant

  if (mode === "primitive") {
    return <ContextMenuPrimitive.Item className={className} {...props} />
  }

  return (
    <ContextMenuPrimitive.Item
      data-slot="context-menu-item"
      data-inset={inset}
      data-variant={resolvedVariant}
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function ContextMenuSubTrigger({
  mode = DEFAULT_MODE,
  className,
  inset,
  children,
  classNameMode = "merge",
  classResolver,
  iconClassName,
  iconClassNameMode = "merge",
  iconClassResolver,
  ...props
}: ContextMenuSubTriggerProps) {
  const defaultClassName =
    "flex cursor-default items-center gap-1.5 rounded-md px-1.5 py-1 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground data-inset:pl-7 data-open:bg-accent data-open:text-accent-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return (
      <ContextMenuPrimitive.SubTrigger className={className} {...props}>
        {children}
        <ChevronRightIcon className={iconClassName} />
      </ContextMenuPrimitive.SubTrigger>
    )
  }

  return (
    <ContextMenuPrimitive.SubTrigger
      data-slot="context-menu-sub-trigger"
      data-inset={inset}
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {children}
      <ChevronRightIcon
        className={resolveStyledContextMenuClassName({
          className: iconClassName,
          defaultClassName: contextMenuClassNames.slot1,
          classNameMode: iconClassNameMode,
          classResolver: iconClassResolver,
        })}
      />
    </ContextMenuPrimitive.SubTrigger>
  )
}

function ContextMenuSubContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: ContextMenuSubContentProps) {
  const defaultClassName =
    "z-50 min-w-32 origin-(--radix-context-menu-content-transform-origin) overflow-hidden rounded-lg border bg-popover p-1 text-popover-foreground shadow-lg duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95"

  if (mode === "primitive") {
    return <ContextMenuPrimitive.SubContent className={className} {...props} />
  }

  return (
    <ContextMenuPrimitive.SubContent
      data-slot="context-menu-sub-content"
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function ContextMenuCheckboxItem({
  mode = DEFAULT_MODE,
  className,
  children,
  checked,
  inset,
  classNameMode = "merge",
  classResolver,
  indicatorClassName,
  indicatorClassNameMode = "merge",
  indicatorClassResolver,
  ...props
}: ContextMenuCheckboxItemProps) {
  const defaultClassName =
    "relative flex cursor-default items-center gap-1.5 rounded-md py-1 pr-8 pl-1.5 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground data-inset:pl-7 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return (
      <ContextMenuPrimitive.CheckboxItem
        className={className}
        checked={checked}
        {...props}
      >
        <span className={indicatorClassName}>
          <ContextMenuPrimitive.ItemIndicator>
            <CheckIcon />
          </ContextMenuPrimitive.ItemIndicator>
        </span>
        {children}
      </ContextMenuPrimitive.CheckboxItem>
    )
  }

  return (
    <ContextMenuPrimitive.CheckboxItem
      data-slot="context-menu-checkbox-item"
      data-inset={inset}
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      checked={checked}
      {...props}
    >
      <span
        className={resolveStyledContextMenuClassName({
          className: indicatorClassName,
          defaultClassName: contextMenuClassNames.slot2,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
      >
        <ContextMenuPrimitive.ItemIndicator>
          <CheckIcon />
        </ContextMenuPrimitive.ItemIndicator>
      </span>
      {children}
    </ContextMenuPrimitive.CheckboxItem>
  )
}

function ContextMenuRadioItem({
  mode = DEFAULT_MODE,
  className,
  children,
  inset,
  classNameMode = "merge",
  classResolver,
  indicatorClassName,
  indicatorClassNameMode = "merge",
  indicatorClassResolver,
  ...props
}: ContextMenuRadioItemProps) {
  const defaultClassName =
    "relative flex cursor-default items-center gap-1.5 rounded-md py-1 pr-8 pl-1.5 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground data-inset:pl-7 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return (
      <ContextMenuPrimitive.RadioItem className={className} {...props}>
        <span className={indicatorClassName}>
          <ContextMenuPrimitive.ItemIndicator>
            <CheckIcon />
          </ContextMenuPrimitive.ItemIndicator>
        </span>
        {children}
      </ContextMenuPrimitive.RadioItem>
    )
  }

  return (
    <ContextMenuPrimitive.RadioItem
      data-slot="context-menu-radio-item"
      data-inset={inset}
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <span
        className={resolveStyledContextMenuClassName({
          className: indicatorClassName,
          defaultClassName: contextMenuClassNames.slot2,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
      >
        <ContextMenuPrimitive.ItemIndicator>
          <CheckIcon />
        </ContextMenuPrimitive.ItemIndicator>
      </span>
      {children}
    </ContextMenuPrimitive.RadioItem>
  )
}

function ContextMenuLabel({
  mode = DEFAULT_MODE,
  className,
  inset,
  classNameMode = "merge",
  classResolver,
  ...props
}: ContextMenuLabelProps) {
  const defaultClassName =
    "px-1.5 py-1 text-xs font-medium text-muted-foreground data-inset:pl-7"

  if (mode === "primitive") {
    return <ContextMenuPrimitive.Label className={className} {...props} />
  }

  return (
    <ContextMenuPrimitive.Label
      data-slot="context-menu-label"
      data-inset={inset}
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function ContextMenuSeparator({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: ContextMenuSeparatorProps) {
  const defaultClassName = "-mx-1 my-1 h-px bg-border"

  if (mode === "primitive") {
    return <ContextMenuPrimitive.Separator className={className} {...props} />
  }

  return (
    <ContextMenuPrimitive.Separator
      data-slot="context-menu-separator"
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function ContextMenuShortcut({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: ContextMenuShortcutProps) {
  const defaultClassName =
    "ml-auto text-xs tracking-widest text-muted-foreground group-focus/context-menu-item:text-accent-foreground"

  if (mode === "primitive") {
    return <span className={className} {...props} />
  }

  return (
    <span
      data-slot="context-menu-shortcut"
      className={resolveStyledContextMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export {
  ContextMenu,
  ContextMenuTrigger,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuCheckboxItem,
  ContextMenuRadioItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuGroup,
  ContextMenuPortal,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuRadioGroup,
}
