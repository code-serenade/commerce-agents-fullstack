import { NavigationMenu as NavigationMenuPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { ChevronDownIcon } from "../../lib/icon-slots"
import {
  navigationMenuClassNames,
  navigationMenuTriggerVariants,
} from "./navigation-menu.styles"
import type {
  NavigationMenuClassResolver,
  NavigationMenuContentProps,
  NavigationMenuIndicatorProps,
  NavigationMenuItemProps,
  NavigationMenuLinkProps,
  NavigationMenuListProps,
  NavigationMenuProps,
  NavigationMenuTriggerProps,
  NavigationMenuViewportProps,
} from "./navigation-menu.types"

function resolveStyledNavigationMenuClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: NavigationMenuClassResolver
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

function NavigationMenu({
  mode = DEFAULT_MODE,
  className,
  children,
  viewport = true,
  classNameMode = "merge",
  classResolver,
  ...props
}: NavigationMenuProps) {
  const defaultClassName =
    "group/navigation-menu relative flex max-w-max flex-1 items-center justify-center"

  if (mode === "primitive") {
    return (
      <NavigationMenuPrimitive.Root className={className} {...props}>
        {children}
        {viewport ? <NavigationMenuViewport mode={mode} /> : null}
      </NavigationMenuPrimitive.Root>
    )
  }

  return (
    <NavigationMenuPrimitive.Root
      data-slot="navigation-menu"
      data-viewport={viewport}
      className={resolveStyledNavigationMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {children}
      {viewport ? <NavigationMenuViewport mode={mode} /> : null}
    </NavigationMenuPrimitive.Root>
  )
}

function NavigationMenuList({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: NavigationMenuListProps) {
  const defaultClassName =
    "group flex flex-1 list-none items-center justify-center gap-0"

  if (mode === "primitive") {
    return <NavigationMenuPrimitive.List className={className} {...props} />
  }

  return (
    <NavigationMenuPrimitive.List
      data-slot="navigation-menu-list"
      className={resolveStyledNavigationMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function NavigationMenuItem({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: NavigationMenuItemProps) {
  const defaultClassName = "relative"

  if (mode === "primitive") {
    return <NavigationMenuPrimitive.Item className={className} {...props} />
  }

  return (
    <NavigationMenuPrimitive.Item
      data-slot="navigation-menu-item"
      className={resolveStyledNavigationMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function NavigationMenuTrigger({
  mode = DEFAULT_MODE,
  className,
  children,
  classNameMode = "merge",
  classResolver,
  iconClassName,
  iconClassNameMode = "merge",
  iconClassResolver,
  ...props
}: NavigationMenuTriggerProps) {
  const defaultClassName = cn(navigationMenuTriggerVariants(), "group")

  if (mode === "primitive") {
    return (
      <NavigationMenuPrimitive.Trigger className={className} {...props}>
        {children}
        <ChevronDownIcon className={iconClassName} aria-hidden="true" />
      </NavigationMenuPrimitive.Trigger>
    )
  }

  return (
    <NavigationMenuPrimitive.Trigger
      data-slot="navigation-menu-trigger"
      className={resolveStyledNavigationMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {children}
      <ChevronDownIcon
        aria-hidden="true"
        className={resolveStyledNavigationMenuClassName({
          className: iconClassName,
          defaultClassName: navigationMenuClassNames.slot1,
          classNameMode: iconClassNameMode,
          classResolver: iconClassResolver,
        })}
      />
    </NavigationMenuPrimitive.Trigger>
  )
}

function NavigationMenuContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: NavigationMenuContentProps) {
  const defaultClassName =
    "top-0 left-0 w-full p-1 ease-[cubic-bezier(0.22,1,0.36,1)] group-data-[viewport=false]/navigation-menu:top-full group-data-[viewport=false]/navigation-menu:mt-1.5 group-data-[viewport=false]/navigation-menu:overflow-hidden group-data-[viewport=false]/navigation-menu:rounded-lg group-data-[viewport=false]/navigation-menu:bg-popover group-data-[viewport=false]/navigation-menu:text-popover-foreground group-data-[viewport=false]/navigation-menu:shadow group-data-[viewport=false]/navigation-menu:ring-1 group-data-[viewport=false]/navigation-menu:ring-foreground/10 group-data-[viewport=false]/navigation-menu:duration-300 data-[motion=from-end]:slide-in-from-right-52 data-[motion=from-start]:slide-in-from-left-52 data-[motion=to-end]:slide-out-to-right-52 data-[motion=to-start]:slide-out-to-left-52 data-[motion^=from-]:animate-in data-[motion^=from-]:fade-in data-[motion^=to-]:animate-out data-[motion^=to-]:fade-out **:data-[slot=navigation-menu-link]:focus:ring-0 **:data-[slot=navigation-menu-link]:focus:outline-none md:absolute md:w-auto group-data-[viewport=false]/navigation-menu:data-open:animate-in group-data-[viewport=false]/navigation-menu:data-open:fade-in-0 group-data-[viewport=false]/navigation-menu:data-open:zoom-in-95 group-data-[viewport=false]/navigation-menu:data-closed:animate-out group-data-[viewport=false]/navigation-menu:data-closed:fade-out-0 group-data-[viewport=false]/navigation-menu:data-closed:zoom-out-95"

  if (mode === "primitive") {
    return <NavigationMenuPrimitive.Content className={className} {...props} />
  }

  return (
    <NavigationMenuPrimitive.Content
      data-slot="navigation-menu-content"
      className={resolveStyledNavigationMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function NavigationMenuViewport({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  containerClassName,
  containerClassNameMode = "merge",
  containerClassResolver,
  ...props
}: NavigationMenuViewportProps) {
  if (mode === "primitive") {
    return <NavigationMenuPrimitive.Viewport className={className} {...props} />
  }

  return (
    <div
      className={resolveStyledNavigationMenuClassName({
        className: containerClassName,
        defaultClassName: navigationMenuClassNames.slot2,
        classNameMode: containerClassNameMode,
        classResolver: containerClassResolver,
      })}
    >
      <NavigationMenuPrimitive.Viewport
        data-slot="navigation-menu-viewport"
        className={resolveStyledNavigationMenuClassName({
          className,
          defaultClassName: navigationMenuClassNames.slot3,
          classNameMode,
          classResolver,
        })}
        {...props}
      />
    </div>
  )
}

function NavigationMenuLink({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: NavigationMenuLinkProps) {
  const defaultClassName =
    "flex items-center gap-2 rounded-lg p-2 text-sm transition-all outline-none hover:bg-muted focus:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-1 in-data-[slot=navigation-menu-content]:rounded-md data-active:bg-muted/50 data-active:hover:bg-muted data-active:focus:bg-muted [&_svg:not([class*='size-'])]:size-4"

  if (mode === "primitive") {
    return <NavigationMenuPrimitive.Link className={className} {...props} />
  }

  return (
    <NavigationMenuPrimitive.Link
      data-slot="navigation-menu-link"
      className={resolveStyledNavigationMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function NavigationMenuIndicator({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  arrowClassName,
  arrowClassNameMode = "merge",
  arrowClassResolver,
  ...props
}: NavigationMenuIndicatorProps) {
  const defaultClassName =
    "top-full z-1 flex h-1.5 items-end justify-center overflow-hidden data-[state=hidden]:animate-out data-[state=hidden]:fade-out data-[state=visible]:animate-in data-[state=visible]:fade-in"

  if (mode === "primitive") {
    return (
      <NavigationMenuPrimitive.Indicator className={className} {...props}>
        <div className={arrowClassName} />
      </NavigationMenuPrimitive.Indicator>
    )
  }

  return (
    <NavigationMenuPrimitive.Indicator
      data-slot="navigation-menu-indicator"
      className={resolveStyledNavigationMenuClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <div
        className={resolveStyledNavigationMenuClassName({
          className: arrowClassName,
          defaultClassName: navigationMenuClassNames.slot4,
          classNameMode: arrowClassNameMode,
          classResolver: arrowClassResolver,
        })}
      />
    </NavigationMenuPrimitive.Indicator>
  )
}

export {
  NavigationMenu,
  NavigationMenuList,
  NavigationMenuItem,
  NavigationMenuContent,
  NavigationMenuTrigger,
  NavigationMenuLink,
  NavigationMenuIndicator,
  NavigationMenuViewport,
}
