"use client"
import { DropdownMenu as DropdownMenuPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { CheckIcon, ChevronRightIcon } from "../../lib/icon-slots"
import { dropdownMenuClassNames } from "./dropdown-menu.styles"
import type {
  DropdownMenuCheckboxItemProps,
  DropdownMenuClassResolver,
  DropdownMenuContentProps,
  DropdownMenuGroupProps,
  DropdownMenuItemProps,
  DropdownMenuItemVariant,
  DropdownMenuLabelProps,
  DropdownMenuPortalProps,
  DropdownMenuProps,
  DropdownMenuRadioGroupProps,
  DropdownMenuRadioItemProps,
  DropdownMenuSeparatorProps,
  DropdownMenuShortcutProps,
  DropdownMenuSubContentProps,
  DropdownMenuSubProps,
  DropdownMenuSubTriggerProps,
  DropdownMenuTriggerProps,
} from "./dropdown-menu.types"

function resolveStyledDropdownMenuClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: DropdownMenuClassResolver
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

function DropdownMenu({ mode = DEFAULT_MODE, ...props }: DropdownMenuProps) {
  if (mode === "primitive") {
    return <DropdownMenuPrimitive.Root {...props} />
  }

  return <DropdownMenuPrimitive.Root data-slot="dropdown-menu" {...props} />
}

function DropdownMenuPortal({
  mode = DEFAULT_MODE,
  ...props
}: DropdownMenuPortalProps) {
  if (mode === "primitive") {
    return <DropdownMenuPrimitive.Portal {...props} />
  }

  return (
    <DropdownMenuPrimitive.Portal data-slot="dropdown-menu-portal" {...props} />
  )
}

function DropdownMenuTrigger({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DropdownMenuTriggerProps) {
  const defaultClassName =
    "outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"

  if (mode === "primitive") {
    return <DropdownMenuPrimitive.Trigger className={className} {...props} />
  }

  return (
    <DropdownMenuPrimitive.Trigger
      data-slot="dropdown-menu-trigger"
      className={resolveStyledDropdownMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DropdownMenuContent({
  mode = DEFAULT_MODE,
  className,
  align = "start",
  sideOffset = 4,
  classNameMode = "merge",
  classResolver,
  ...props
}: DropdownMenuContentProps) {
  const defaultClassName =
    "z-50 max-h-(--radix-dropdown-menu-content-available-height) w-(--radix-dropdown-menu-trigger-width) min-w-32 origin-(--radix-dropdown-menu-content-transform-origin) overflow-x-hidden overflow-y-auto rounded-xl bg-popover p-1 text-popover-foreground shadow-md ring-1 ring-foreground/10 duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:overflow-hidden data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95"

  if (mode === "primitive") {
    return (
      <DropdownMenuPrimitive.Portal>
        <DropdownMenuPrimitive.Content
          sideOffset={sideOffset}
          align={align}
          className={className}
          {...props}
        />
      </DropdownMenuPrimitive.Portal>
    )
  }

  return (
    <DropdownMenuPrimitive.Portal>
      <DropdownMenuPrimitive.Content
        data-slot="dropdown-menu-content"
        sideOffset={sideOffset}
        align={align}
        className={resolveStyledDropdownMenuClassName({
          className,
          defaultClassName,
          classNameMode,
          classResolver,
        })}
        {...props}
      />
    </DropdownMenuPrimitive.Portal>
  )
}

function DropdownMenuGroup({
  mode = DEFAULT_MODE,
  ...props
}: DropdownMenuGroupProps) {
  if (mode === "primitive") {
    return <DropdownMenuPrimitive.Group {...props} />
  }

  return (
    <DropdownMenuPrimitive.Group data-slot="dropdown-menu-group" {...props} />
  )
}

function DropdownMenuItem({
  mode = DEFAULT_MODE,
  className,
  inset,
  variant = "default",
  classNameMode = "merge",
  classResolver,
  ...props
}: DropdownMenuItemProps) {
  const defaultClassName =
    "group/dropdown-menu-item relative flex min-h-8 cursor-default items-center gap-2 rounded-md px-2 py-1.5 text-sm leading-none outline-hidden select-none focus:bg-accent focus:text-accent-foreground not-data-[variant=destructive]:focus:**:text-accent-foreground data-inset:pl-7 data-[variant=destructive]:text-destructive data-[variant=destructive]:focus:bg-destructive/10 data-[variant=destructive]:focus:text-destructive dark:data-[variant=destructive]:focus:bg-destructive/20 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 data-[variant=destructive]:*:[svg]:text-destructive"
  const resolvedVariant = (variant ?? "default") as DropdownMenuItemVariant

  if (mode === "primitive") {
    return <DropdownMenuPrimitive.Item className={className} {...props} />
  }

  return (
    <DropdownMenuPrimitive.Item
      data-slot="dropdown-menu-item"
      data-inset={inset}
      data-variant={resolvedVariant}
      className={resolveStyledDropdownMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DropdownMenuCheckboxItem({
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
}: DropdownMenuCheckboxItemProps) {
  const defaultClassName =
    "relative flex min-h-8 cursor-default items-center gap-2 rounded-md py-1.5 pr-8 pl-2 text-sm leading-none outline-hidden select-none focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-inset:pl-7 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return (
      <DropdownMenuPrimitive.CheckboxItem
        className={className}
        checked={checked}
        {...props}
      >
        <span className={indicatorClassName}>
          <DropdownMenuPrimitive.ItemIndicator>
            <CheckIcon />
          </DropdownMenuPrimitive.ItemIndicator>
        </span>
        {children}
      </DropdownMenuPrimitive.CheckboxItem>
    )
  }

  return (
    <DropdownMenuPrimitive.CheckboxItem
      data-slot="dropdown-menu-checkbox-item"
      data-inset={inset}
      className={resolveStyledDropdownMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      checked={checked}
      {...props}
    >
      <span
        className={resolveStyledDropdownMenuClassName({
          className: indicatorClassName,
          defaultClassName: dropdownMenuClassNames.slot1,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
        data-slot="dropdown-menu-checkbox-item-indicator"
      >
        <DropdownMenuPrimitive.ItemIndicator>
          <CheckIcon />
        </DropdownMenuPrimitive.ItemIndicator>
      </span>
      {children}
    </DropdownMenuPrimitive.CheckboxItem>
  )
}

function DropdownMenuRadioGroup({
  mode = DEFAULT_MODE,
  ...props
}: DropdownMenuRadioGroupProps) {
  if (mode === "primitive") {
    return <DropdownMenuPrimitive.RadioGroup {...props} />
  }

  return (
    <DropdownMenuPrimitive.RadioGroup
      data-slot="dropdown-menu-radio-group"
      {...props}
    />
  )
}

function DropdownMenuRadioItem({
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
}: DropdownMenuRadioItemProps) {
  const defaultClassName =
    "relative flex min-h-8 cursor-default items-center gap-2 rounded-md py-1.5 pr-8 pl-2 text-sm leading-none outline-hidden select-none focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-inset:pl-7 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return (
      <DropdownMenuPrimitive.RadioItem className={className} {...props}>
        <span className={indicatorClassName}>
          <DropdownMenuPrimitive.ItemIndicator>
            <CheckIcon />
          </DropdownMenuPrimitive.ItemIndicator>
        </span>
        {children}
      </DropdownMenuPrimitive.RadioItem>
    )
  }

  return (
    <DropdownMenuPrimitive.RadioItem
      data-slot="dropdown-menu-radio-item"
      data-inset={inset}
      className={resolveStyledDropdownMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <span
        className={resolveStyledDropdownMenuClassName({
          className: indicatorClassName,
          defaultClassName: dropdownMenuClassNames.slot1,
          classNameMode: indicatorClassNameMode,
          classResolver: indicatorClassResolver,
        })}
        data-slot="dropdown-menu-radio-item-indicator"
      >
        <DropdownMenuPrimitive.ItemIndicator>
          <CheckIcon />
        </DropdownMenuPrimitive.ItemIndicator>
      </span>
      {children}
    </DropdownMenuPrimitive.RadioItem>
  )
}

function DropdownMenuLabel({
  mode = DEFAULT_MODE,
  className,
  inset,
  classNameMode = "merge",
  classResolver,
  ...props
}: DropdownMenuLabelProps) {
  const defaultClassName =
    "px-2 py-1 text-xs font-medium text-muted-foreground data-inset:pl-7"

  if (mode === "primitive") {
    return <DropdownMenuPrimitive.Label className={className} {...props} />
  }

  return (
    <DropdownMenuPrimitive.Label
      data-slot="dropdown-menu-label"
      data-inset={inset}
      className={resolveStyledDropdownMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DropdownMenuSeparator({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DropdownMenuSeparatorProps) {
  const defaultClassName = "-mx-1 my-1 h-px bg-border"

  if (mode === "primitive") {
    return <DropdownMenuPrimitive.Separator className={className} {...props} />
  }

  return (
    <DropdownMenuPrimitive.Separator
      data-slot="dropdown-menu-separator"
      className={resolveStyledDropdownMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DropdownMenuShortcut({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DropdownMenuShortcutProps) {
  const defaultClassName =
    "ml-auto text-xs tracking-widest text-muted-foreground group-focus/dropdown-menu-item:text-accent-foreground"

  if (mode === "primitive") {
    return <span className={className} {...props} />
  }

  return (
    <span
      data-slot="dropdown-menu-shortcut"
      className={resolveStyledDropdownMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function DropdownMenuSub({
  mode = DEFAULT_MODE,
  ...props
}: DropdownMenuSubProps) {
  if (mode === "primitive") {
    return <DropdownMenuPrimitive.Sub {...props} />
  }

  return <DropdownMenuPrimitive.Sub data-slot="dropdown-menu-sub" {...props} />
}

function DropdownMenuSubTrigger({
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
}: DropdownMenuSubTriggerProps) {
  const defaultClassName =
    "flex cursor-default items-center gap-1.5 rounded-md px-1.5 py-1 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground not-data-[variant=destructive]:focus:**:text-accent-foreground data-inset:pl-7 data-open:bg-accent data-open:text-accent-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return (
      <DropdownMenuPrimitive.SubTrigger className={className} {...props}>
        {children}
        <ChevronRightIcon className={iconClassName} />
      </DropdownMenuPrimitive.SubTrigger>
    )
  }

  return (
    <DropdownMenuPrimitive.SubTrigger
      data-slot="dropdown-menu-sub-trigger"
      data-inset={inset}
      className={resolveStyledDropdownMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {children}
      <ChevronRightIcon
        className={resolveStyledDropdownMenuClassName({
          className: iconClassName,
          defaultClassName: dropdownMenuClassNames.slot2,
          classNameMode: iconClassNameMode,
          classResolver: iconClassResolver,
        })}
      />
    </DropdownMenuPrimitive.SubTrigger>
  )
}

function DropdownMenuSubContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: DropdownMenuSubContentProps) {
  const defaultClassName =
    "z-50 min-w-24 origin-(--radix-dropdown-menu-content-transform-origin) overflow-hidden rounded-lg bg-popover p-1 text-popover-foreground shadow-lg ring-1 ring-foreground/10 duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95"

  if (mode === "primitive") {
    return <DropdownMenuPrimitive.SubContent className={className} {...props} />
  }

  return (
    <DropdownMenuPrimitive.SubContent
      data-slot="dropdown-menu-sub-content"
      className={resolveStyledDropdownMenuClassName({
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
  DropdownMenu,
  DropdownMenuPortal,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuItem,
  DropdownMenuCheckboxItem,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
}
