"use client"

import { Menubar as MenubarPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { CheckIcon, ChevronRightIcon } from "../../lib/icon-slots"
import { menubarClassNames } from "./menubar.styles"
import type {
  MenubarCheckboxItemProps,
  MenubarClassResolver,
  MenubarContentProps,
  MenubarGroupProps,
  MenubarItemProps,
  MenubarItemVariant,
  MenubarLabelProps,
  MenubarMenuProps,
  MenubarPortalProps,
  MenubarProps,
  MenubarRadioGroupProps,
  MenubarRadioItemProps,
  MenubarSeparatorProps,
  MenubarShortcutProps,
  MenubarSubContentProps,
  MenubarSubProps,
  MenubarSubTriggerProps,
  MenubarTriggerProps,
} from "./menubar.types"

function resolveStyledMenubarClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: MenubarClassResolver
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

function Menubar({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: MenubarProps) {
  const defaultClassName =
    "flex h-8 items-center gap-0.5 rounded-lg border p-0.75"

  if (mode === "primitive") {
    return <MenubarPrimitive.Root className={className} {...props} />
  }

  return (
    <MenubarPrimitive.Root
      data-slot="menubar"
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function MenubarMenu({ mode = DEFAULT_MODE, ...props }: MenubarMenuProps) {
  if (mode === "primitive") {
    return <MenubarPrimitive.Menu {...props} />
  }

  return <MenubarPrimitive.Menu data-slot="menubar-menu" {...props} />
}

function MenubarGroup({ mode = DEFAULT_MODE, ...props }: MenubarGroupProps) {
  if (mode === "primitive") {
    return <MenubarPrimitive.Group {...props} />
  }

  return <MenubarPrimitive.Group data-slot="menubar-group" {...props} />
}

function MenubarPortal({ mode = DEFAULT_MODE, ...props }: MenubarPortalProps) {
  if (mode === "primitive") {
    return <MenubarPrimitive.Portal {...props} />
  }

  return <MenubarPrimitive.Portal data-slot="menubar-portal" {...props} />
}

function MenubarRadioGroup({
  mode = DEFAULT_MODE,
  ...props
}: MenubarRadioGroupProps) {
  if (mode === "primitive") {
    return <MenubarPrimitive.RadioGroup {...props} />
  }

  return (
    <MenubarPrimitive.RadioGroup data-slot="menubar-radio-group" {...props} />
  )
}

function MenubarTrigger({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: MenubarTriggerProps) {
  const defaultClassName =
    "flex items-center rounded-sm px-1.5 py-0.5 text-sm font-medium outline-hidden select-none hover:bg-muted aria-expanded:bg-muted"

  if (mode === "primitive") {
    return <MenubarPrimitive.Trigger className={className} {...props} />
  }

  return (
    <MenubarPrimitive.Trigger
      data-slot="menubar-trigger"
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function MenubarContent({
  mode = DEFAULT_MODE,
  className,
  align = "start",
  alignOffset = -4,
  sideOffset = 8,
  classNameMode = "merge",
  classResolver,
  ...props
}: MenubarContentProps) {
  const defaultClassName =
    "z-50 min-w-36 origin-(--radix-menubar-content-transform-origin) overflow-hidden rounded-lg bg-popover p-1 text-popover-foreground shadow-md ring-1 ring-foreground/10 duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95"

  if (mode === "primitive") {
    return (
      <MenubarPrimitive.Portal>
        <MenubarPrimitive.Content
          align={align}
          alignOffset={alignOffset}
          sideOffset={sideOffset}
          className={className}
          {...props}
        />
      </MenubarPrimitive.Portal>
    )
  }

  return (
    <MenubarPrimitive.Portal>
      <MenubarPrimitive.Content
        data-slot="menubar-content"
        align={align}
        alignOffset={alignOffset}
        sideOffset={sideOffset}
        className={resolveStyledMenubarClassName({
          className,
          defaultClassName,
          classNameMode,
          classResolver,
        })}
        {...props}
      />
    </MenubarPrimitive.Portal>
  )
}

function MenubarItem({
  mode = DEFAULT_MODE,
  className,
  inset,
  variant = "default",
  classNameMode = "merge",
  classResolver,
  ...props
}: MenubarItemProps) {
  const defaultClassName =
    "group/menubar-item relative flex cursor-default items-center gap-1.5 rounded-md px-1.5 py-1 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground not-data-[variant=destructive]:focus:**:text-accent-foreground data-inset:pl-7 data-[variant=destructive]:text-destructive data-[variant=destructive]:focus:bg-destructive/10 data-[variant=destructive]:focus:text-destructive dark:data-[variant=destructive]:focus:bg-destructive/20 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 data-[variant=destructive]:*:[svg]:text-destructive!"
  const resolvedVariant = (variant ?? "default") as MenubarItemVariant

  if (mode === "primitive") {
    return <MenubarPrimitive.Item className={className} {...props} />
  }

  return (
    <MenubarPrimitive.Item
      data-slot="menubar-item"
      data-inset={inset}
      data-variant={resolvedVariant}
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function MenubarCheckboxItem({
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
}: MenubarCheckboxItemProps) {
  const defaultClassName =
    "relative flex cursor-default items-center gap-1.5 rounded-md py-1 pr-1.5 pl-7 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-inset:pl-7 data-disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0"

  if (mode === "primitive") {
    return (
      <MenubarPrimitive.CheckboxItem
        className={className}
        checked={checked}
        {...props}
      >
        <span className={indicatorClassName}>
          <MenubarPrimitive.ItemIndicator>
            <CheckIcon />
          </MenubarPrimitive.ItemIndicator>
        </span>
        {children}
      </MenubarPrimitive.CheckboxItem>
    )
  }

  return (
    <MenubarPrimitive.CheckboxItem
      data-slot="menubar-checkbox-item"
      data-inset={inset}
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      checked={checked}
      {...props}
    >
      <span
        className={resolveStyledMenubarClassName({
          className: indicatorClassName,
          defaultClassName: menubarClassNames.slot1,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
      >
        <MenubarPrimitive.ItemIndicator>
          <CheckIcon />
        </MenubarPrimitive.ItemIndicator>
      </span>
      {children}
    </MenubarPrimitive.CheckboxItem>
  )
}

function MenubarRadioItem({
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
}: MenubarRadioItemProps) {
  const defaultClassName =
    "relative flex cursor-default items-center gap-1.5 rounded-md py-1 pr-1.5 pl-7 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-inset:pl-7 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return (
      <MenubarPrimitive.RadioItem className={className} {...props}>
        <span className={indicatorClassName}>
          <MenubarPrimitive.ItemIndicator>
            <CheckIcon />
          </MenubarPrimitive.ItemIndicator>
        </span>
        {children}
      </MenubarPrimitive.RadioItem>
    )
  }

  return (
    <MenubarPrimitive.RadioItem
      data-slot="menubar-radio-item"
      data-inset={inset}
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <span
        className={resolveStyledMenubarClassName({
          className: indicatorClassName,
          defaultClassName: menubarClassNames.slot1,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
      >
        <MenubarPrimitive.ItemIndicator>
          <CheckIcon />
        </MenubarPrimitive.ItemIndicator>
      </span>
      {children}
    </MenubarPrimitive.RadioItem>
  )
}

function MenubarLabel({
  mode = DEFAULT_MODE,
  className,
  inset,
  classNameMode = "merge",
  classResolver,
  ...props
}: MenubarLabelProps) {
  const defaultClassName = "px-1.5 py-1 text-sm font-medium data-inset:pl-7"

  if (mode === "primitive") {
    return <MenubarPrimitive.Label className={className} {...props} />
  }

  return (
    <MenubarPrimitive.Label
      data-slot="menubar-label"
      data-inset={inset}
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function MenubarSeparator({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: MenubarSeparatorProps) {
  const defaultClassName = "-mx-1 my-1 h-px bg-border"

  if (mode === "primitive") {
    return <MenubarPrimitive.Separator className={className} {...props} />
  }

  return (
    <MenubarPrimitive.Separator
      data-slot="menubar-separator"
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function MenubarShortcut({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: MenubarShortcutProps) {
  const defaultClassName =
    "ml-auto text-xs tracking-widest text-muted-foreground group-focus/menubar-item:text-accent-foreground"

  if (mode === "primitive") {
    return <span className={className} {...props} />
  }

  return (
    <span
      data-slot="menubar-shortcut"
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function MenubarSub({ mode = DEFAULT_MODE, ...props }: MenubarSubProps) {
  if (mode === "primitive") {
    return <MenubarPrimitive.Sub {...props} />
  }

  return <MenubarPrimitive.Sub data-slot="menubar-sub" {...props} />
}

function MenubarSubTrigger({
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
}: MenubarSubTriggerProps) {
  const defaultClassName =
    "flex cursor-default items-center gap-1.5 rounded-md px-1.5 py-1 text-sm outline-none select-none focus:bg-accent focus:text-accent-foreground data-inset:pl-7 data-open:bg-accent data-open:text-accent-foreground [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return (
      <MenubarPrimitive.SubTrigger className={className} {...props}>
        {children}
        <ChevronRightIcon className={iconClassName} />
      </MenubarPrimitive.SubTrigger>
    )
  }

  return (
    <MenubarPrimitive.SubTrigger
      data-slot="menubar-sub-trigger"
      data-inset={inset}
      className={resolveStyledMenubarClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {children}
      <ChevronRightIcon
        className={resolveStyledMenubarClassName({
          className: iconClassName,
          defaultClassName: menubarClassNames.slot2,
          classNameMode: iconClassNameMode,
          classResolver: iconClassResolver,
        })}
      />
    </MenubarPrimitive.SubTrigger>
  )
}

function MenubarSubContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: MenubarSubContentProps) {
  const defaultClassName =
    "z-50 min-w-32 origin-(--radix-menubar-content-transform-origin) overflow-hidden rounded-lg bg-popover p-1 text-popover-foreground shadow-lg ring-1 ring-foreground/10 duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95"

  if (mode === "primitive") {
    return <MenubarPrimitive.SubContent className={className} {...props} />
  }

  return (
    <MenubarPrimitive.SubContent
      data-slot="menubar-sub-content"
      className={resolveStyledMenubarClassName({
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
  Menubar,
  MenubarPortal,
  MenubarMenu,
  MenubarTrigger,
  MenubarContent,
  MenubarGroup,
  MenubarSeparator,
  MenubarLabel,
  MenubarItem,
  MenubarShortcut,
  MenubarCheckboxItem,
  MenubarRadioGroup,
  MenubarRadioItem,
  MenubarSub,
  MenubarSubTrigger,
  MenubarSubContent,
}
